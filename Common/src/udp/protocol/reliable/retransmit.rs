use std::cmp::max;
use std::collections::{BTreeMap, HashMap, HashSet, LinkedList};
use std::ops::{Rem, Sub};
use std::time::{Duration, Instant};

use lru::LruCache;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};
use serde::{Deserialize, Serialize};

use crate::udp::protocol::{DisconnectedStatus, FrameBuiltListener, FrameReceivedListener};
use crate::udp::protocol::congestion::CongestionControl;
use crate::udp::protocol::frame::{Frame, FrameId};
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::reliable::ack::header::AckFrameHeader;
use crate::udp::protocol::reliable::statistics::RetransmitStatistics;

///
/// Повторная посылка фреймов, для которых не пришло подтверждение
///
#[cfg_attr(test, automock)]
pub trait Retransmitter {
	///
	/// Установить время ожидания ответа на посланный фрейм
	///
	fn set_ack_wait_duration(&mut self, duration: Duration);
	
	///
	/// Получить процент излишне повторно отправленных пакетов
	///
	fn get_redundant_frames_percent(&mut self, now: &Instant) -> Option<f64>;
}


pub struct RetransmitterImpl {
	///
	/// Фреймы, отсортированные по времени отсылки
	///
	frames: LinkedList<ScheduledFrame>,
	
	///
	/// Фреймы, для которых мы ожидаем ASK
	///
	unacked_frames: HashSet<FrameId>,
	
	///
	/// Текущее максимальное количество повтора пакета
	///
	max_retransmit_count: u8,
	
	///
	/// Связь повторно отправленных фреймов с оригинальными (для обработки ASK ответов)
	///
	retransmit_to_original: LruCache<FrameId, FrameId>,
	
	///
	/// Время ожидания подтверждения на фрейм
	///
	ack_wait_duration: Duration,
	
	statistics: RetransmitStatistics,
}

#[derive(Debug)]
pub struct ScheduledFrame {
	pub time: Instant,
	pub original_frame_id: FrameId,
	pub frame: Frame,
	pub retransmit_count: u8,
}


///
/// Заголовок для указания факта повторной передачи данного фрейма
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RetransmitFrameHeader {
	pub original_frame_id: FrameId,
	pub retransmit_count: u8,
}

impl RetransmitFrameHeader {
	fn new(original_frame_id: FrameId, retransmit_count: u8) -> Self {
		Self {
			original_frame_id,
			retransmit_count,
		}
	}
}


impl Default for RetransmitterImpl {
	fn default() -> Self {
		Self {
			frames: Default::default(),
			unacked_frames: Default::default(),
			max_retransmit_count: Default::default(),
			retransmit_to_original: LruCache::new(RetransmitterImpl::RETRANSMIT_TO_ORIGINAL_MAX_LEN),
			ack_wait_duration: RetransmitterImpl::DEFAULT_ACK_TIMEOUT,
			statistics: Default::default(),
		}
	}
}

impl RetransmitterImpl {
	///
	/// Количество повторных пересылок фрейма, после которого соединение будет считаться разорванным
	/// примерно 30 секунд без подтверждения при [DEFAULT_ASK_TIMEOUT]
	///
	pub const RETRANSMIT_LIMIT: u8 = 100;
	
	///
	/// Время ожидания ACK по умолчанию
	///
	pub const DEFAULT_ACK_TIMEOUT: Duration = Duration::from_millis(300);
	
	///
	/// Максимальная длина [self.retransmit_to_original]
	///
	/// - должно гарантированно хватить все фреймы, ожидающие ASK
	/// - включая ASK на повторно отосланные фреймы
	///
	pub const RETRANSMIT_TO_ORIGINAL_MAX_LEN: usize = 4096;
	
	
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
						scheduled_frame.time = now.clone();
						
						let original_frame_id = scheduled_frame.original_frame_id;
						let mut retransmit_frame = scheduled_frame.frame.clone();
						retransmit_frame.header.frame_id = retransmit_frame_id;
						let retransmit_header = Header::RetransmitFrame(RetransmitFrameHeader::new(original_frame_id, retransmit_count));
						retransmit_frame.headers.add(retransmit_header);
						
						self.retransmit_to_original.put(retransmit_frame_id, original_frame_id);
						
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
	
	
	fn schedule_retransmit(&mut self, frame: Frame, original_frame_id: FrameId, retransmit_count: u8, now: &Instant) {
		self.frames.push_back(ScheduledFrame {
			time: now.clone(),
			original_frame_id,
			frame,
			retransmit_count,
		});
	}
}


impl Retransmitter for RetransmitterImpl {
	fn set_ack_wait_duration(&mut self, duration: Duration) {
		self.ack_wait_duration = duration;
	}
	
	fn get_redundant_frames_percent(&mut self, now: &Instant) -> Option<f64> {
		let redundant = self.statistics.get_average_redundant_frames(now);
		let retransmit = self.statistics.get_average_retransmit_frames(now);
		redundant.zip(retransmit).map(|(redundant, retransmit)| {
			redundant as f64 / retransmit as f64
		})
	}
}


impl FrameReceivedListener for RetransmitterImpl {
	///
	/// Обрабатываем подтверждения фреймов
	///
	fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		let ack_headers: Vec<&AckFrameHeader> = frame.headers.find(Header::predicate_AckFrame);
		ack_headers.iter().for_each(|ack_header| {
			ack_header.get_frames().iter().for_each(|frame_id| {
				let original_frame_id = match self.retransmit_to_original.get(frame_id) {
					None => {
						*frame_id
					}
					Some(original_frame_id) => {
						*original_frame_id
					}
				};
				
				!self.unacked_frames.remove(&original_frame_id);
				self.statistics.on_ack_received(*frame_id, original_frame_id, now);
			})
		});
	}
}

impl FrameBuiltListener for RetransmitterImpl {
	///
	/// Фрейм отослан - запоминаем для повтора
	///
	fn on_frame_built(&mut self, frame: &Frame, now: &Instant) {
		if frame.is_reliability() {
			let original_grame_id = frame.header.frame_id;
			let mut frame = frame.clone();
			frame.commands.unreliability.clear();
			self.schedule_retransmit(
				frame,
				original_grame_id,
				0,
				now,
			);
			self.unacked_frames.insert(original_grame_id);
		}
	}
}


impl DisconnectedStatus for RetransmitterImpl {
	fn disconnected(&mut self, now: &Instant) -> bool {
		self.max_retransmit_count >= RetransmitterImpl::RETRANSMIT_LIMIT
	}
}


#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;
	
	use crate::udp::protocol::{DisconnectedStatus, FrameBuiltListener, FrameReceivedListener};
	use crate::udp::protocol::frame::{Frame, FrameId};
	use crate::udp::protocol::frame::applications::ApplicationCommand;
	use crate::udp::protocol::frame::headers::Header;
	use crate::udp::protocol::reliable::ack::header::AckFrameHeader;
	use crate::udp::protocol::reliable::retransmit::RetransmitterImpl;
	
	#[test]
	///
	/// Если не было отосланных фреймов - то нет фреймов и для повтора
	///
	fn should_empty_when_get_retransmit_frame() {
		let mut handler = RetransmitterImpl::default();
		assert!(matches!(handler.get_retransmit_frame(&Instant::now(), 1), Option::None));
	}
	
	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_empty_when_no_timeout() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		handler.on_frame_built(&create_reliability_frame(1), &now);
		assert!(matches!(handler.get_retransmit_frame(&now,2), Option::None));
	}
	
	///
	/// Для повторно отправляемого фрейма должен быть добавлен заголовок с id исходного фрейма
	///
	#[test]
	fn should_add_retransmit_header() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let original_frame = create_reliability_frame(1);
		handler.on_frame_built(&original_frame, &now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::Some(frame)
				if frame.header.frame_id == 2
				&&
				frame.headers.first(Header::predicate_RetransmitFrame).unwrap().original_frame_id==original_frame.header.frame_id
			)
		);
	}
	
	
	///
	/// Для фрейма не получено подтверждение, таймаут ожидания прошел
	///
	#[test]
	fn should_return_retransmit_frame_when_timeout() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id ==2 )
		);
	}
	
	///
	/// Для фрейма без надежной доставки не должно быть повторных фреймов
	///
	#[test]
	fn should_return_none_for_unreliable_frame() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_unreliable_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::None)
		);
	}
	
	///
	/// Если для фрейма получен ASK - то его не должно быть в повторных
	///
	#[test]
	fn should_return_none_then_ack() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		handler.on_frame_received(&create_ack_frame(100, frame.header.frame_id), &now);
		let get_time = now.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::None)
		);
	}
	
	///
	/// Если для фрейма получен ASK на повторно отправленный фрейм - то его не должно быть в повторных
	///
	#[test]
	fn should_return_none_then_ack_retransmitted_frame() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		let get_time = now.add(handler.ack_wait_duration);
		let retransmit_frame = handler.get_retransmit_frame(&get_time, 2).unwrap();
		
		handler.on_frame_received(&create_ack_frame(100, retransmit_frame.header.frame_id), &now);
		let get_time = get_time.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,3),
				Option::None)
		);
	}
	
	
	///
	/// Если не было ASK после повторной отправки - то фрейм должен быть перепослан через Timeout
	///
	#[test]
	fn should_retransmit_after_retransmit() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,2),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id == 2)
		);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,3),
				Option::None )
		);
		let get_time = get_time.add(handler.ack_wait_duration);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time,4),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id == 4 )
		);
	}
	
	
	///
	/// Канал должен быть закрыт, после N не успешных попыток отправок
	///
	#[test]
	fn should_close_after_fail_retransmits() {
		let mut handler = RetransmitterImpl::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let mut get_time = now.clone();
		for _ in 0..RetransmitterImpl::RETRANSMIT_LIMIT - 1 {
			get_time = get_time.add(handler.ack_wait_duration);
			handler.get_retransmit_frame(&get_time, 2);
		}
		
		assert_eq!(handler.disconnected(&get_time), false);
		
		get_time = get_time.add(handler.ack_wait_duration);
		handler.get_retransmit_frame(&get_time, 3);
		
		assert_eq!(handler.disconnected(&get_time), true);
	}
	
	
	///
	/// В повторно отправленном фрейме не должно быть команд с ненадежной доставкой
	///
	#[test]
	fn should_delete_unreliable_commands_for_retransmit_frame() {
		let mut handler = RetransmitterImpl::default();
		let mut frame = create_reliability_frame(1);
		frame.commands.unreliability.push(ApplicationCommand::Ping("".to_string()));
		let now = Instant::now();
		handler.on_frame_built(&frame, &now);
		
		let now = now.add(handler.ack_wait_duration);
		assert!(matches!(handler.get_retransmit_frame(&now,2), Option::Some(frame) if frame.commands.unreliability.is_empty()))
	}
	
	
	fn create_reliability_frame(frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame.commands.reliability.push(ApplicationCommand::Ping("".to_string()));
		frame
	}
	
	fn create_unreliable_frame(frame_id: FrameId) -> Frame {
		Frame::new(frame_id)
	}
	
	fn create_ack_frame(frame_id: FrameId, acked_frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame.headers.add(Header::AckFrame(AckFrameHeader::new(acked_frame_id)));
		frame
	}
}