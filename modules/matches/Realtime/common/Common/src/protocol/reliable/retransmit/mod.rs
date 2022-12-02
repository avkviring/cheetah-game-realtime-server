use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use std::ops::Sub;
use std::time::{Duration, Instant};

use fnv::FnvBuildHasher;

use crate::protocol::frame::headers::{Header, HeaderVec};
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::FrameId;
use crate::protocol::reliable::ack::header::AckHeader;
use crate::protocol::reliable::retransmit::header::RetransmitHeader;
use crate::protocol::reliable::statistics::RetransmitStatistics;

pub mod header;

///
/// Количество фреймов с командами, требующими надежную доставку в секунду
///
pub const RELIABILITY_FRAME_PER_SECOND: usize = 10;

///
/// Время ожидания доставки оригинально фрейма (при повторных пересылках)
///
pub const RETRANSMIT_MAX_TIME_IN_SEC: usize = 10;

///
/// Время ожидания ACK
///
pub const RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC: f64 = 0.5;

///
/// Количество повторных пересылок фрейма, после которого соединение будет считаться разорванным
///
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub const RETRANSMIT_LIMIT: u8 = (RETRANSMIT_MAX_TIME_IN_SEC as f64 / RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC) as u8;

///
/// количество фреймов в буферах, должно гарантированно хватить для всех фреймов
/// как только количество фреймов будет больше - то канал переходит в состояние disconnected
///
pub const RETRANSMIT_FRAMES_CAPACITY: usize = RELIABILITY_FRAME_PER_SECOND * RETRANSMIT_MAX_TIME_IN_SEC;

#[derive(Debug)]
pub struct Retransmit {
	///
	/// Фреймы, отсортированные по времени отсылки
	///
	frames: VecDeque<ScheduledFrame>,

	///
	/// Фреймы, для которых мы ожидаем ACK
	///
	wait_ack_frames: HashSet<FrameId, FnvBuildHasher>,

	///
	/// Текущее максимальное количество повтора пакета
	///
	max_retransmit_count: u8,
	///
	/// Время ожидания подтверждения на фрейм
	///
	ack_wait_duration: Duration,

	pub statistics: RetransmitStatistics,
}

#[derive(Debug)]
pub struct ScheduledFrame {
	pub time: Instant,
	pub original_frame_id: FrameId,
	pub frame: OutFrame,
	pub retransmit_count: u8,
}

impl Default for Retransmit {
	fn default() -> Self {
		Self {
			frames: Default::default(),
			wait_ack_frames: Default::default(),
			max_retransmit_count: Default::default(),
			ack_wait_duration: Duration::from_secs_f64(RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC),
			statistics: Default::default(),
		}
	}
}

impl Retransmit {
	///
	/// Получить фрейм для повторной отправки (если такой есть)
	/// - метод необходимо вызывать пока результат Some
	///
	#[allow(clippy::unwrap_in_result)]
	pub fn get_retransmit_frame(&mut self, now: Instant, retransmit_frame_id: FrameId) -> Option<OutFrame> {
		loop {
			match self.frames.front() {
				None => {
					return None;
				}
				Some(scheduled_frame) => {
					if !self.wait_ack_frames.contains(&scheduled_frame.original_frame_id) {
						self.frames.pop_front();
					} else if now.sub(scheduled_frame.time) >= self.ack_wait_duration {
						let mut scheduled_frame = self.frames.pop_front().unwrap();

						let retransmit_count = scheduled_frame.retransmit_count + 1;
						self.max_retransmit_count = max(self.max_retransmit_count, retransmit_count);
						scheduled_frame.retransmit_count = retransmit_count;
						scheduled_frame.time = now;

						let original_frame_id = scheduled_frame.original_frame_id;
						let mut retransmit_frame = scheduled_frame.frame.clone();
						retransmit_frame.frame_id = retransmit_frame_id;
						let retransmit_header = Header::Retransmit(RetransmitHeader::new(original_frame_id, retransmit_count));
						retransmit_frame.headers.add(retransmit_header);

						// мы только-что удалили фрейм, значит место в точно должно быть
						// поэтому unwrap вполне ок
						self.frames.push_back(scheduled_frame);

						self.statistics.on_retransmit_frame(now);
						return Some(retransmit_frame);
					} else {
						return None;
					}
				}
			}
		}
	}

	///
	/// Обрабатываем подтверждения фреймов
	///
	pub(crate) fn on_frame_received(&mut self, frame: &InFrame, now: Instant) {
		let ack_headers: HeaderVec<&AckHeader> = frame.headers.find(Header::predicate_ack);
		ack_headers.iter().for_each(|ack_header| {
			ack_header.get_frames().for_each(|frame_id| {
				self.wait_ack_frames.remove(frame_id);
				self.statistics.on_ack_received(*frame_id, now);
			});
		});
	}
	///
	/// Фрейм отослан - запоминаем для повтора
	///
	pub fn build_frame(&mut self, frame: &OutFrame, now: Instant) {
		if frame.contains_reliability_command() {
			let original_frame_id = frame.frame_id;
			let mut reliable_frame = OutFrame::new(original_frame_id);
			reliable_frame.headers = frame.headers.clone();
			frame.get_commands().filter(|c| c.channel.is_reliable()).for_each(|c| {
				reliable_frame.add_command(c.clone());
			});

			self.frames.push_back(ScheduledFrame {
				time: now,
				original_frame_id,
				frame: reliable_frame,
				retransmit_count: 0,
			});

			self.wait_ack_frames.insert(original_frame_id);
		}
	}

	#[must_use]
	pub fn disconnected(&self, _: Instant) -> bool {
		self.max_retransmit_count >= RETRANSMIT_LIMIT
			|| self.frames.len() > RETRANSMIT_FRAMES_CAPACITY
			|| self.wait_ack_frames.len() > RETRANSMIT_FRAMES_CAPACITY
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::event::EventCommand;
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::frame::output::OutFrame;
	use crate::protocol::frame::FrameId;
	use crate::protocol::reliable::ack::header::AckHeader;
	use crate::protocol::reliable::retransmit::{Retransmit, RETRANSMIT_LIMIT};

	#[test]
	///
	/// Если не было отосланных фреймов - то нет фреймов и для повтора
	///
	fn should_empty_when_get_retransmit_frame() {
		let mut handler = Retransmit::default();
		assert!(matches!(handler.get_retransmit_frame(Instant::now(), 1), None));
	}

	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_empty_when_no_timeout() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		handler.build_frame(&create_reliability_frame(1), now);
		assert!(matches!(handler.get_retransmit_frame(now, 2), None));
	}

	///
	/// Для повторно отправляемого фрейма должен быть добавлен заголовок с id исходного фрейма
	///
	#[test]
	fn should_add_retransmit_header() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let original_frame = create_reliability_frame(1);
		handler.build_frame(&original_frame, now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
			handler.get_retransmit_frame(get_time,2),
			Some(frame)
			if frame.frame_id == 2
			&&
			frame.headers.first(Header::predicate_retransmit).unwrap().original_frame_id==original_frame.frame_id
		));
	}

	///
	/// Для фрейма не получено подтверждение, таймаут ожидания прошел
	///
	#[test]
	fn should_return_retransmit_frame_when_timeout() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(get_time,2),
				Some(retransmit_frame) if retransmit_frame.frame_id ==2 ));
	}

	///
	/// Для фрейма без надежной доставки не должно быть повторных фреймов
	///
	#[test]
	fn should_return_none_for_unreliable_frame() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let frame = create_unreliable_frame(1);
		handler.build_frame(&frame, now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(get_time, 2), None));
	}

	///
	/// Если для фрейма получен ACK - то его не должно быть в повторных
	///
	#[test]
	fn should_return_none_then_ack() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);
		handler.on_frame_received(&create_ack_frame(100, frame.frame_id), now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(get_time, 2), None));
	}

	///
	/// Если не было ACK после повторной отправки - то фрейм должен быть повторно отослан через
	/// Timeout
	///
	#[test]
	fn should_retransmit_after_retransmit() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(get_time,2),
				Some(retransmit_frame) if retransmit_frame.frame_id == 2));
		assert!(matches!(handler.get_retransmit_frame(get_time, 3), None));
		let get_time = get_time.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(get_time,4),
				Some(retransmit_frame) if retransmit_frame.frame_id == 4 ));
	}

	///
	/// Канал должен быть закрыт, после N не успешных попыток отправок
	///
	#[test]
	fn should_close_after_fail_retransmits() {
		let mut handler = Retransmit::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);

		let mut get_time = now;
		for _ in 0..RETRANSMIT_LIMIT - 1 {
			get_time = get_time.add(handler.ack_wait_duration);
			handler.get_retransmit_frame(get_time, 2);
		}

		assert!(!handler.disconnected(get_time));

		get_time = get_time.add(handler.ack_wait_duration);
		handler.get_retransmit_frame(get_time, 3);

		assert!(handler.disconnected(get_time));
	}

	///
	/// В повторно отправленном фрейме не должно быть команд с ненадежной доставкой
	///
	#[test]
	fn should_delete_unreliable_commands_for_retransmit_frame() {
		let mut handler = Retransmit::default();
		let mut frame = OutFrame::new(0);
		frame.add_command(CommandWithChannel {
			channel: Channel::UnreliableUnordered,
			both_direction_command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		});

		let reliable_command = CommandWithChannel {
			channel: Channel::ReliableUnordered,
			both_direction_command: BothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			})),
		};
		frame.add_command(reliable_command.clone());
		let now = Instant::now();
		handler.build_frame(&frame, now);
		let now = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(now,2),
			Some(frame)
			if *frame.get_commands().as_slice()==[reliable_command]));
	}

	fn create_reliability_frame(frame_id: FrameId) -> OutFrame {
		let mut frame = OutFrame::new(frame_id);
		frame.add_command(CommandWithChannel {
			channel: Channel::ReliableUnordered,
			both_direction_command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		});
		frame
	}

	fn create_unreliable_frame(frame_id: FrameId) -> OutFrame {
		OutFrame::new(frame_id)
	}

	fn create_ack_frame(frame_id: FrameId, acked_frame_id: FrameId) -> InFrame {
		let mut frame = InFrame::new(frame_id, Default::default(), Default::default());
		let mut ack_header = AckHeader::default();
		ack_header.add_frame_id(acked_frame_id);
		frame.headers.add(Header::Ack(ack_header));
		frame
	}
}
