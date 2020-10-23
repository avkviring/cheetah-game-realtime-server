use std::cmp::max;
use std::collections::{HashSet, LinkedList};
use std::ops::Sub;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::udp::protocol::{DisconnectedStatus, FrameBuiltListener, FrameReceivedListener};
use crate::udp::protocol::congestion::CongestionControl;
use crate::udp::protocol::frame::{Frame, FrameId};
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::reliable::ask::header::AskFrameHeader;

///
/// Повторная посылка фреймов, для которых не пришло подтверждение
///
pub struct Retransmitter {
	///
	/// Фреймы, отсортированные по времени отсылки
	///
	pub frames: LinkedList<FrameAndTime>,
	
	///
	/// Фреймы, для которых мы ожидаем ASK
	///
	pub wait_ask_frames: HashSet<FrameId>,
	
	///
	/// Максимальное количество повтора пакета
	///
	pub max_retransmit_count: u8,
	
	pub timeout: Duration,
}

pub struct FrameAndTime {
	pub time: Instant,
	pub frame: Frame,
	pub retransmit_count: u8,
}

///
/// Заголовок для указания факта повторной передачи данного фрейма
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RetransmitMarkHeader {
	pub retransmit_count: u8
}


impl Default for Retransmitter {
	fn default() -> Self {
		Self {
			frames: Default::default(),
			wait_ask_frames: Default::default(),
			max_retransmit_count: 0,
			timeout: CongestionControl::DEFAULT_ASK_TIMEOUT,
		}
	}
}

impl Retransmitter {
	pub const RETRANSMIT_LIMIT: u8 = 10;
	///
	/// Получить фрейм для повторной отправки (если такой есть)
	///
	pub fn get_retransmit_frame(&mut self, now: &Instant) -> Option<Frame> {
		match self.frames.front() {
			None => {
				Option::None
			}
			Some(frame_and_time) => {
				if !self.wait_ask_frames.contains(&frame_and_time.frame.header.frame_id) {
					self.frames.pop_front();
					Option::None
				} else if now.sub(frame_and_time.time) >= self.timeout {
					let mut frame_and_time = self.frames.pop_front().unwrap();
					let retransmit_count = frame_and_time.retransmit_count + 1;
					self.max_retransmit_count = max(self.max_retransmit_count, retransmit_count);
					let mut frame_to_send = frame_and_time.frame;
					self.frames.push_back(FrameAndTime {
						time: now.clone(),
						frame: frame_to_send.clone(),
						retransmit_count,
					});
					frame_to_send.headers.add(Header::RetransmitMark(RetransmitMarkHeader { retransmit_count }));
					Option::Some(frame_to_send)
				} else {
					Option::None
				}
			}
		}
	}
	
	fn create_retransmit_frame(source: &Frame) -> Frame {
		let mut frame = source.clone();
		frame.commands.unreliability.clear();
		frame
	}
}


impl FrameReceivedListener for Retransmitter {
	///
	/// Обрабатываем подтверждения фреймов
	///
	fn on_frame_received(&mut self, frame: &Frame, _: &Instant) {
		let ask_headers: Vec<&AskFrameHeader> = frame.headers.find(Header::predicate_AskFrame);
		ask_headers.iter().for_each(|ask_header| {
			ask_header.get_frames().iter().for_each(|frame_id| {
				self.wait_ask_frames.remove(frame_id);
			})
		});
	}
}

impl FrameBuiltListener for Retransmitter {
	///
	/// Фрейм отослан - запоминаем для повтора
	///
	fn on_frame_built(&mut self, frame: &Frame, now: &Instant) {
		if !frame.commands.reliability.is_empty() {
			self.frames.push_back(FrameAndTime {
				time: now.clone(),
				frame: Retransmitter::create_retransmit_frame(frame),
				retransmit_count: 0,
			});
			self.wait_ask_frames.insert(frame.header.frame_id);
		}
	}
}


impl DisconnectedStatus for Retransmitter {
	fn disconnected(&mut self, now: &Instant) -> bool {
		self.max_retransmit_count >= Retransmitter::RETRANSMIT_LIMIT
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
	use crate::udp::protocol::reliable::ask::header::AskFrameHeader;
	use crate::udp::protocol::reliable::retransmit::Retransmitter;
	
	#[test]
	///
	/// Если не было отосланных фреймов - то нет фреймов и для повтора
	///
	fn should_empty_when_get_retransmit_frame() {
		let mut handler = Retransmitter::default();
		assert!(matches!(handler.get_retransmit_frame(&Instant::now()), Option::None));
	}
	
	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_empty_when_no_timeout() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		handler.on_frame_built(&create_reliability_frame(1), &now);
		assert!(matches!(handler.get_retransmit_frame(&now), Option::None));
	}
	
	///
	/// Для фрейма не получено подтверждение, но таймаут ожидания еще не прошел
	///
	#[test]
	fn should_add_retransmit_mark() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		handler.on_frame_built(&create_reliability_frame(1), &now);
		let get_time = now.add(handler.timeout);
		
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::Some(frame) if frame.headers.first(Header::predicate_RetransmitMark).is_some()
			)
		);
	}
	
	
	///
	/// Для фрейма не получено подтверждение, таймаут ожидания прошел
	///
	#[test]
	fn should_return_retransmit_frame_when_timeout() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.timeout);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id ==frame.header.frame_id )
		);
	}
	
	///
	/// Для фрейма без надежной доставки не должно быть повторных фреймов
	///
	#[test]
	fn should_return_none_for_unreliable_frame() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_unreliable_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.timeout);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::None)
		);
	}
	
	///
	/// Если для фрейма получен ASK - то его не должно быть в повторных
	///
	#[test]
	fn should_return_none_then_ask() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		handler.on_frame_received(&create_ask_frame(100, frame.header.frame_id), &now);
		let get_time = now.add(handler.timeout);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::None)
		);
	}
	
	///
	/// Если не было ASK после повторной отправки - то фрейм должен быть перепослан через Timeout
	///
	#[test]
	fn should_retransmit_after_retransmit() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let get_time = now.add(handler.timeout);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id ==frame.header.frame_id )
		);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::None )
		);
		let get_time = get_time.add(handler.timeout);
		assert!(
			matches!(
				handler.get_retransmit_frame(&get_time),
				Option::Some(retransmit_frame) if retransmit_frame.header.frame_id ==frame.header.frame_id )
		);
	}
	
	
	///
	/// Канал должен быть закрыт, после N не успешных попыток отправок
	///
	#[test]
	fn should_close_after_fail_retransmits() {
		let mut handler = Retransmitter::default();
		let now = Instant::now();
		let frame = create_reliability_frame(1);
		handler.on_frame_built(&frame, &now);
		
		let mut get_time = now.clone();
		for _ in 0..Retransmitter::RETRANSMIT_LIMIT - 1 {
			get_time = get_time.add(handler.timeout);
			handler.get_retransmit_frame(&get_time);
		}
		
		assert_eq!(handler.disconnected(&get_time), false);
		
		get_time = get_time.add(handler.timeout);
		handler.get_retransmit_frame(&get_time);
		
		assert_eq!(handler.disconnected(&get_time), true);
	}
	
	
	///
	/// В повторно отправленном фрейме не должно быть команд с ненадежной доставкой
	///
	#[test]
	fn should_delete_unreliable_commands_for_retransmit_frame() {
		let mut handler = Retransmitter::default();
		let mut frame = create_reliability_frame(1);
		frame.commands.unreliability.push(ApplicationCommand::Ping("".to_string()));
		let now = Instant::now();
		handler.on_frame_built(&frame, &now);
		
		let now = now.add(handler.timeout);
		assert!(matches!(handler.get_retransmit_frame(&now), Option::Some(frame) if frame.commands.unreliability.is_empty()))
	}
	
	
	fn create_reliability_frame(frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame.commands.reliability.push(ApplicationCommand::Ping("".to_string()));
		frame
	}
	
	fn create_unreliable_frame(frame_id: FrameId) -> Frame {
		Frame::new(frame_id)
	}
	
	fn create_ask_frame(frame_id: FrameId, asked_frame_id: FrameId) -> Frame {
		let mut frame = Frame::new(frame_id);
		frame.headers.add(Header::AskFrame(AskFrameHeader::new(asked_frame_id)));
		frame
	}
}