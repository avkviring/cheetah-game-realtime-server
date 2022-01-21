use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use std::ops::Sub;
use std::time::{Duration, Instant};

use fnv::FnvBuildHasher;

use crate::protocol::frame::{Frame, FrameId, MAX_COMMAND_IN_FRAME};
use crate::protocol::frame::headers::{Header, HeaderVec};
use crate::protocol::reliable::ack::header::AckHeader;
use crate::protocol::reliable::retransmit::header::RetransmitHeader;
use crate::protocol::reliable::statistics::RetransmitStatistics;

pub mod header;

///
/// Количество фреймов с командами, требующими надежную доставку в секунду
///
pub const RELIABILITY_FRAME_PER_SECOND:usize = 10;

///
/// Время ожидания доставки оригинально фрейма (при повторных пересылках)
///
pub const RETRANSMIT_MAX_TIME_IN_SEC:usize = 10;

///
/// Время ожидания ACK
///
pub const RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC:f64 = 0.5;

///
/// Количество повторных пересылок фрейма, после которого соединение будет считаться разорванным
///
pub const RETRANSMIT_LIMIT: usize =
	(RETRANSMIT_MAX_TIME_IN_SEC as f64/RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC) as usize;

///
/// количество фреймов в буферах, должно гарантированно хватить для всех фреймов
/// как только количество фреймов будет больше - то канал переходит в состояние disconnected
///
pub const RETRANSMIT_FRAMES_CAPACITY:usize = RELIABILITY_FRAME_PER_SECOND*RETRANSMIT_MAX_TIME_IN_SEC;



#[derive(Debug)]
pub struct Retransmitter {

	///
	/// Фреймы, отсортированные по времени отсылки
	///
	frames: VecDeque<ScheduledFrame>,

	///
	/// Фреймы, для которых мы ожидаем ACK
	///
	unacked_frames: HashSet<FrameId, FnvBuildHasher>,

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
	pub frame: Frame,
	pub retransmit_count: u8,
}


impl Default for Retransmitter {
	fn default() -> Self {
		Self {
			frames: VecDeque::with_capacity(MAX_COMMAND_IN_FRAME),
			unacked_frames: HashSet::with_capacity_and_hasher(MAX_COMMAND_IN_FRAME, FnvBuildHasher::default()),
			max_retransmit_count: Default::default(),
			ack_wait_duration: Duration::from_secs_f64(RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC),
			statistics: Default::default(),
		}
	}
}



impl Retransmitter {

	///
	/// Получить фрейм для повторной отправки (если такой есть)
	/// - метод необходимо вызывать пока результат Option::Some
	///
	pub fn get_retransmit_frame(&mut self, now: &Instant, retransmit_frame_id: FrameId) -> Option<Frame> {
		loop {
			match self.frames.front() {
				None => {
					return Option::None;
				}
				Some(scheduled_frame) => {
					if !self.unacked_frames.contains(&scheduled_frame.original_frame_id) {
						self.frames.pop_front();
					} else if now.sub(scheduled_frame.time) >= self.ack_wait_duration {
						let mut scheduled_frame = self.frames.pop_front().unwrap();

						let retransmit_count = scheduled_frame.retransmit_count + 1;
						self.max_retransmit_count = max(self.max_retransmit_count, retransmit_count);
						scheduled_frame.retransmit_count = retransmit_count;
						scheduled_frame.time = *now;

						let original_frame_id = scheduled_frame.original_frame_id;
						let mut retransmit_frame = scheduled_frame.frame.clone();
						retransmit_frame.frame_id = retransmit_frame_id;
						let retransmit_header = Header::Retransmit(RetransmitHeader::new(original_frame_id, retransmit_count));
						retransmit_frame.headers.add(retransmit_header);

						// мы только-что удалили фрейм, значит место в точно должно быть
						// поэтому unwrap вполне ок
						self.frames.push_back(scheduled_frame);

						self.statistics.on_retransmit_frame(now);
						return Option::Some(retransmit_frame);
					} else {
						return Option::None;
					}
				}
			}
		}
	}

	fn set_ack_wait_duration(&mut self, duration: Duration) {
		self.ack_wait_duration = duration;
		log::info!("[retransmit] set_ack_wait_duration({:?})", duration);
	}

	fn get_redundant_frames_percent(&mut self, now: &Instant) -> Option<f64> {
		let redundant = self.statistics.get_average_redundant_frames(now);
		let retransmit = self.statistics.get_average_retransmit_frames(now);
		redundant
			.zip(retransmit)
			.map(|(redundant, retransmit)| redundant as f64 / retransmit as f64)
	}

	///
	/// Обрабатываем подтверждения фреймов
	///
	pub(crate) fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		let ack_headers: HeaderVec<&AckHeader> = frame.headers.find(Header::predicate_ack);
		ack_headers.iter().for_each(|ack_header| {
			ack_header.get_frames().iter().for_each(|frame_id| {
				self.unacked_frames.remove(frame_id);
				self.statistics.on_ack_received(*frame_id, now);
			});
		});
	}
	///
	/// Фрейм отослан - запоминаем для повтора
	///
	pub fn build_frame(&mut self, frame: &Frame, now: &Instant) {
		if frame.is_reliability() {
			let original_frame_id = frame.frame_id;
			let mut cloned_frame = frame.clone();
			cloned_frame.commands.clear();
			for command in frame.commands.iter() {
				if !command.channel.is_reliable() {
					continue;
				}
				cloned_frame.commands.push(command.clone()).unwrap();
			}

			self.frames.push_back(ScheduledFrame {
				time: *now,
				original_frame_id,
				frame: cloned_frame,
				retransmit_count: 0
			});

			self.unacked_frames.insert(original_frame_id);
		}
	}

	pub fn disconnected(&self, _: &Instant) -> bool {
		self.max_retransmit_count >= RETRANSMIT_LIMIT as u8
			|| self.frames.len() > RETRANSMIT_FRAMES_CAPACITY
			|| self.unacked_frames.len() >RETRANSMIT_FRAMES_CAPACITY
	}
}



#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::event::EventCommand;
	use crate::protocol::frame::{Frame, FrameId};
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::reliable::ack::header::AckHeader;
	use crate::protocol::reliable::retransmit::{RETRANSMIT_LIMIT, Retransmitter};

	#[test]
	///
	/// Если не было отосланных фреймов - то нет фреймов и для повтора
	///
	fn should_empty_when_get_retransmit_frame() {
		let mut handler = Retransmitter::default();
		assert!(matches!(handler.get_retransmit_frame(&Instant::now(), 1), Option::None));
	}

	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_empty_when_no_timeout() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		handler.build_frame(&create_reliability_frame(1), &now);
		assert!(matches!(handler.get_retransmit_frame(&now, 2), Option::None));
	}

	///
	/// Для повторно отправляемого фрейма должен быть добавлен заголовок с id исходного фрейма
	///
	#[test]
	fn should_add_retransmit_header() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let original_frame = create_reliability_frame(1);
		handler.build_frame(&original_frame, &now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
			handler.get_retransmit_frame(&get_time,2),
			Option::Some(frame)
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
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, &now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::Some(retransmit_frame) if retransmit_frame.frame_id ==2 ));
	}

	///
	/// Для фрейма без надежной доставки не должно быть повторных фреймов
	///
	#[test]
	fn should_return_none_for_unreliable_frame() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_unreliable_frame(1);
		handler.build_frame(&frame, &now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(&get_time, 2), Option::None));
	}

	///
	/// Если для фрейма получен ACK - то его не должно быть в повторных
	///
	#[test]
	fn should_return_none_then_ack() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, &now);
		handler.on_frame_received(&create_ack_frame(100, frame.frame_id), &now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(&get_time, 2), Option::None));
	}

	///
	/// Если не было ACK после повторной отправки - то фрейм должен быть повторно отослан через
	/// Timeout
	///
	#[test]
	fn should_retransmit_after_retransmit() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, &now);

		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::Some(retransmit_frame) if retransmit_frame.frame_id == 2));
		assert!(matches!(handler.get_retransmit_frame(&get_time, 3), Option::None));
		let get_time = get_time.add(handler.ack_wait_duration);
		assert!(matches!(
				handler.get_retransmit_frame(&get_time,4),
				Option::Some(retransmit_frame) if retransmit_frame.frame_id == 4 ));
	}

	///
	/// Канал должен быть закрыт, после N не успешных попыток отправок
	///
	#[test]
	fn should_close_after_fail_retransmits() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, &now);

		let mut get_time = now;
		for _ in 0..RETRANSMIT_LIMIT - 1 {
			get_time = get_time.add(handler.ack_wait_duration);
			handler.get_retransmit_frame(&get_time, 2);
		}

		assert!(!handler.disconnected(&get_time));

		get_time = get_time.add(handler.ack_wait_duration);
		handler.get_retransmit_frame(&get_time, 3);

		assert!(handler.disconnected(&get_time));
	}

	///
	/// В повторно отправленном фрейме не должно быть команд с ненадежной доставкой
	///
	#[test]
	fn should_delete_unreliable_commands_for_retransmit_frame() {
		let mut handler = Retransmitter::default();
		let mut frame = Frame::new(0);
		frame
			.commands
			.push(CommandWithChannel {
				channel: Channel::UnreliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			})
			.unwrap();

		let reliable_command = CommandWithChannel {
			channel: Channel::ReliableUnordered,
			command: BothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			})),
		};
		frame
			.commands
			.push(reliable_command.clone())
			.unwrap();
		let now = Instant::now();
		handler.build_frame(&frame, &now);
		let now = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(&now,2), 
			Option::Some(frame) 
			if *frame.commands.as_slice()==[reliable_command]));
	}

	fn create_reliability_frame(frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame
			.commands
			.push(CommandWithChannel {
				channel: Channel::ReliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			})
			.unwrap();
		frame
	}

	fn create_unreliable_frame(frame_id: FrameId) -> Frame {
		Frame::new(frame_id)
	}

	fn create_ack_frame(frame_id: FrameId, acked_frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame.headers.add(Header::Ack(AckHeader::new(acked_frame_id)));
		frame
	}
}
