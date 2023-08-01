use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use std::ops::Sub;
use std::time::{Duration, Instant};
use std::usize;

use fnv::FnvBuildHasher;

use crate::frame::headers::{Header, HeaderVec};
use crate::frame::Frame;
use crate::frame::FrameId;
use crate::reliable::ack::header::AckHeader;
use crate::reliable::retransmit::header::RetransmitHeader;

pub mod header;

///
/// Время ожидания ACK
///
pub const RETRANSMIT_DEFAULT_ACK_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Debug)]
pub struct Retransmitter {
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
	current_max_retransmit_count: usize,
	///
	/// Время ожидания подтверждения на фрейм
	///
	ack_wait_duration: Duration,

	retransmit_limit: usize,
}

#[derive(Debug)]
pub struct ScheduledFrame {
	pub time: Instant,
	pub original_frame_id: FrameId,
	pub frame: Frame,
	pub retransmit_count: usize,
}

impl Retransmitter {
	pub(crate) fn new(disconnect_timeout: Duration) -> Self {
		let retransmit_limit = (disconnect_timeout.as_secs_f64() / RETRANSMIT_DEFAULT_ACK_TIMEOUT.as_secs_f64()) as usize;

		Self {
			frames: Default::default(),
			wait_ack_frames: Default::default(),
			current_max_retransmit_count: Default::default(),
			ack_wait_duration: RETRANSMIT_DEFAULT_ACK_TIMEOUT,
			retransmit_limit,
		}
	}
	///
	/// Получить фрейм для повторной отправки (если такой есть)
	/// - метод необходимо вызывать пока результат Some
	///
	pub fn get_retransmit_frame(&mut self, now: Instant, retransmit_frame_id: FrameId) -> Option<Frame> {
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

						let retransmit_count = scheduled_frame.retransmit_count.checked_add(1).unwrap_or(usize::MAX);
						if retransmit_count == usize::MAX {
							tracing::error!("Retransmit count overflow");
						}

						self.current_max_retransmit_count = max(self.current_max_retransmit_count, retransmit_count);
						scheduled_frame.retransmit_count = retransmit_count;
						scheduled_frame.time = now;

						let original_frame_id = scheduled_frame.original_frame_id;
						let mut retransmit_frame = scheduled_frame.frame.clone();
						retransmit_frame.frame_id = retransmit_frame_id;
						let retransmit_header = Header::Retransmit(RetransmitHeader::new(original_frame_id));
						retransmit_frame.headers.add(retransmit_header);
						self.frames.push_back(scheduled_frame);
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
	pub(crate) fn on_frame_received(&mut self, frame: &Frame) {
		let ack_headers: HeaderVec<&AckHeader> = frame.headers.find(Header::predicate_ack);
		ack_headers.iter().for_each(|ack_header| {
			ack_header.get_frames().for_each(|frame_id| {
				self.wait_ack_frames.remove(frame_id);
			});
		});
	}
	///
	/// Фрейм отослан - запоминаем для повтора
	///
	pub fn build_frame(&mut self, frame: &Frame, now: Instant) {
		if frame.reliability {
			let original_frame_id = frame.frame_id;
			self.frames.push_back(ScheduledFrame {
				time: now,
				original_frame_id,
				frame: frame.clone(),
				retransmit_count: 0,
			});
			self.wait_ack_frames.insert(original_frame_id);
		}
	}

	pub fn is_disconnected(&self, _: Instant) -> bool {
		self.current_max_retransmit_count >= self.retransmit_limit
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::frame::headers::Header;
	use crate::frame::Frame;
	use crate::frame::FrameId;
	use crate::reliable::ack::header::AckHeader;
	use crate::reliable::retransmit::Retransmitter;

	#[test]
	///
	/// Если не было отосланных фреймов - то нет фреймов и для повтора
	///
	fn should_empty_when_get_retransmit_frame() {
		let mut handler = Retransmitter::new(Duration::default());
		assert!(matches!(handler.get_retransmit_frame(Instant::now(), 1), None));
	}

	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_empty_when_no_timeout() {
		let mut handler = Retransmitter::new(Duration::default());
		let now = Instant::now();
		handler.build_frame(&create_reliability_frame(1), now);
		assert!(matches!(handler.get_retransmit_frame(now, 2), None));
	}

	///
	/// Для повторно отправляемого фрейма должен быть добавлен заголовок с id исходного фрейма
	///
	#[test]
	fn should_add_retransmit_header() {
		let mut handler = Retransmitter::new(Duration::default());
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
		let mut handler = Retransmitter::new(Duration::default());
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
		let mut handler = Retransmitter::new(Duration::default());
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
		let mut handler = Retransmitter::new(Duration::default());
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);
		handler.on_frame_received(&create_ack_frame(100, frame.frame_id));
		let get_time = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(get_time, 2), None));
	}

	///
	/// Если не было ACK после повторной отправки - то фрейм должен быть повторно отослан через
	/// Timeout
	///
	#[test]
	fn should_retransmit_after_retransmit() {
		let mut handler = Retransmitter::new(Duration::default());
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
	fn should_disconnet_by_timeout() {
		let mut handler = Retransmitter::new(Duration::from_secs(1));
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.build_frame(&frame, now);

		let mut get_time = now;
		for _ in 0..handler.retransmit_limit - 1 {
			get_time = get_time.add(handler.ack_wait_duration);
			handler.get_retransmit_frame(get_time, 2);
		}

		assert!(!handler.is_disconnected(get_time));

		get_time = get_time.add(handler.ack_wait_duration);
		handler.get_retransmit_frame(get_time, 3);

		assert!(handler.is_disconnected(get_time));
	}

	fn create_reliability_frame(frame_id: FrameId) -> Frame {
		Frame::new(0, frame_id, true, Default::default())
	}

	fn create_unreliable_frame(frame_id: FrameId) -> Frame {
		Frame::new(0, frame_id, false, Default::default())
	}

	fn create_ack_frame(frame_id: FrameId, acked_frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(0, frame_id, false, Default::default());
		let mut ack_header = AckHeader::default();
		ack_header.add_frame_id(acked_frame_id);
		frame.headers.add(Header::Ack(ack_header));
		frame
	}
}
