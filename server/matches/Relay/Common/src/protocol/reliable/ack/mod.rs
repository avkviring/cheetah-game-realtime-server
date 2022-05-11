use std::ops::Add;
use std::time::{Duration, Instant};

use crate::protocol::frame::headers::Header;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::FrameId;
use crate::protocol::reliable::ack::header::AckHeader;

pub mod header;

///
/// Управление рассылкой подтверждения о приеме пакетов
/// Каждое оповещение рассылается N раз, с временными интервалами между посылками, для избежания
/// потери сообщение из-за временной недоступности сети
///
#[derive(Debug, Default)]
pub struct AckSender {
	ack_tasks: heapless::Vec<AckTask, 100>,
}

#[derive(Debug, Copy, Clone)]
struct AckTask {
	frame_id: FrameId,
	ack_count: u8,
	scheduled_ack: Instant,
}

impl AckSender {
	///
	/// Минимальное количество отсылок подтверждения для одного пакета
	///
	pub const MAX_ACK_FOR_FRAME: u8 = 3;

	///
	/// Интервал между посылками подтверждений
	///
	pub const SEND_INTERVAL: Duration = Duration::from_millis(10);
}

impl AckSender {
	pub fn contains_self_data(&self, now: &Instant) -> bool {
		self.ack_tasks.iter().any(|t| *now >= t.scheduled_ack)
	}

	///
	/// В каждый исходящий пакет добавляем id полученных пакетов
	/// Необходимо обеспечить многократную посылку информации о полученных фреймах, так как
	/// одиночная посылка может не дойди то получателя, это приведет к долгому ожиданию и
	/// повторной отсылки пакета
	///
	///
	pub fn build_out_frame(&mut self, frame: &mut OutFrame, now: &Instant) {
		let mut header = AckHeader::default();
		self.ack_tasks.iter_mut().for_each(|task| {
			if *now >= task.scheduled_ack && !header.is_full() {
				header.add_frame_id(task.frame_id);
				task.ack_count += 1;
				task.scheduled_ack = now.add(AckSender::SEND_INTERVAL)
			}
		});

		let cloned_tasks = self.ack_tasks.clone();
		self.ack_tasks.clear();
		cloned_tasks
			.into_iter()
			.filter(|t| t.ack_count < AckSender::MAX_ACK_FOR_FRAME)
			.for_each(|t| {
				self.ack_tasks.push(t).unwrap();
			});
		frame.headers.add(Header::Ack(header));
	}

	pub fn on_frame_received(&mut self, frame: &InFrame, now: &Instant) {
		if frame.contains_reliability_command() {
			if let Err(_) = self.ack_tasks.push(AckTask {
				frame_id: frame.get_original_frame_id(),
				ack_count: 0,
				scheduled_ack: *now,
			}) {
				tracing::error!("AckSender overflow ack_tasks",);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;

	use crate::commands::c2s::C2SCommand;

	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::frame::output::OutFrame;
	use crate::protocol::reliable::ack::header::AckHeader;
	use crate::protocol::reliable::ack::AckSender;

	#[test]
	///
	/// Если не было входящих пакетов - то и не должно быть исходящих ack пакетов
	///
	fn should_ack_not_need_send() {
		let ack_sender = AckSender::default();
		assert!(!ack_sender.contains_self_data(&Instant::now()));
	}

	#[test]
	fn should_ack() {
		let mut now = Instant::now();
		let mut ack_sender = AckSender::default();
		let in_frame = InFrame::new(10, Default::default(), [create_command()].into_iter().collect());
		ack_sender.on_frame_received(&in_frame, &now);

		for _ in 0..AckSender::MAX_ACK_FOR_FRAME {
			// в исходящем фрейме должно быть подтверждение на входящий
			assert!(ack_sender.contains_self_data(&now));
			let header = build_out_frame(&mut now, &mut ack_sender);
			assert!(header.get_frames().any(|id| *id == in_frame.frame_id));
			// отослали - теперь подтверждения быть не должно
			assert!(!ack_sender.contains_self_data(&now));
			let header = build_out_frame(&mut now, &mut ack_sender);
			assert!(!header.get_frames().any(|id| *id == in_frame.frame_id));
			now = now.add(AckSender::SEND_INTERVAL);
		}

		// теперь данных быть не должно, так как мы их отослали необходимое количество раз
		assert!(!ack_sender.contains_self_data(&now));
		let header = build_out_frame(&mut now, &mut ack_sender);
		assert!(!header.get_frames().any(|id| *id == in_frame.frame_id));
	}

	fn build_out_frame(now: &mut Instant, ack_sender: &mut AckSender) -> AckHeader {
		let out_frame = &mut OutFrame::new(200);
		ack_sender.build_out_frame(out_frame, now);
		let header: &AckHeader = out_frame.headers.first(Header::predicate_ack).unwrap();
		header.clone()
	}

	fn create_command() -> CommandWithChannel {
		CommandWithChannel {
			channel: Channel::ReliableUnordered,
			both_direction_command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		}
	}
}
