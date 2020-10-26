use std::ops::Add;
use std::time::{Duration, Instant};

use crate::udp::protocol::{FrameBuilder, FrameReceivedListener, NOT_EXIST_FRAME_ID};
use crate::udp::protocol::frame::{Frame, FrameId};
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::reliable::ask::header::AskFrameHeader;

pub mod header;


///
/// Управление рассылкой подтверждения о приеме пакетов
/// - подтверждается [frame.header.frame_id], а не [frame.get_original_id()]
///
pub struct AskSender {
	///
	/// Кольцевой буфер c frame_id,
	/// для всех фреймов из данного буфера производится отсылка ACK пакетов
	///
	pub frames: [FrameId; AskSender::BUFFER_SIZE],
	///
	/// Позиция для следующего фрейма
	///
	pub next_frame_position: usize,
	
	///
	/// Количество отправленных ACK пакетов для frame_id из [frames]
	/// Необходимо для сбора статистики и контроля достаточности количества отправленных ACK
	///
	pub ack_counts: [u8; AskSender::BUFFER_SIZE],
	
	///
	/// Время следующей отправки ACK
	///
	pub schedule_send: Option<Instant>,
	
	///
	/// Количество фреймов для которых ACK отправлен недостаточное количество раз
	///
	pub low_count_ack_count: u64,
}


impl Default for AskSender {
	fn default() -> Self {
		Self {
			frames: [NOT_EXIST_FRAME_ID; AskSender::BUFFER_SIZE],
			next_frame_position: 0,
			ack_counts: [0; AskSender::BUFFER_SIZE],
			schedule_send: Option::None,
			low_count_ack_count: 0,
		}
	}
}

impl AskSender {
	///
	/// Время, с которого может быть запрошена отправка пакета с подтверждением
	///
	pub const SCHEDULE_SEND_TIME: Duration = Duration::from_millis(1);
	
	
	///
	/// Количество подтверждаемых фреймов для одного исходящего пакета
	///
	pub const BUFFER_SIZE: usize = 64;
	
	pub const ALERT_LOW_COUNT_ACK: u8 = 2;
}


impl FrameBuilder for AskSender {
	fn contains_self_data(&self, now: &Instant) -> bool {
		match self.schedule_send {
			None => {
				false
			}
			Some(ref time_to_sent) => {
				now >= time_to_sent
			}
		}
	}
	
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		self.schedule_send = Option::None;
		let mut frames = self.frames.clone();
		frames.sort();
		
		let mut current_header: Option<AskFrameHeader> = Option::None;
		for i in 0..frames.len() {
			let frame_id = frames[i];
			if frame_id == NOT_EXIST_FRAME_ID {
				continue;
			}
			self.ack_counts[i] += 1;
			match current_header {
				None => {
					let header = AskFrameHeader::new(frame_id);
					current_header = Option::Some(header);
				}
				Some(ref mut header) => {
					if !header.store_frame_id(frame_id) {
						frame.headers.add(Header::AskFrame(header.clone()));
						let header = AskFrameHeader::new(frame_id);
						current_header = Option::Some(header);
					}
				}
			}
		}
		
		match current_header {
			None => {}
			Some(header) => {
				frame.headers.add(Header::AskFrame(header));
			}
		}
	}
}

impl FrameReceivedListener for AskSender {
	fn on_frame_received(&mut self, frame: &Frame, now: &Instant) {
		// если нет reliability команд - то подтверждать не надо
		if !frame.commands.reliability.is_empty() {
			let frame_id = frame.header.frame_id;
			
			// записываем frame_id в буфер
			// так как буфер кольцевой - то мы перепишем старый frame_id
			// если подтверждение для старого frame_id отправлялось недостаточное количество раз - то отметим данный факт
			let frame_position = self.next_frame_position;
			if self.ack_counts[frame_position] < AskSender::ALERT_LOW_COUNT_ACK && self.frames[frame_position] != NOT_EXIST_FRAME_ID {
				self.low_count_ack_count += 1;
			}
			self.frames[frame_position] = frame_id;
			self.ack_counts[frame_position] = 0;
			
			
			self.next_frame_position += 1;
			if self.next_frame_position > AskSender::BUFFER_SIZE {
				self.next_frame_position = 0;
			}
			
			self.schedule_send = Option::Some(now.add(AskSender::SCHEDULE_SEND_TIME));
		}
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;
	
	use crate::udp::protocol::{FrameBuilder, FrameReceivedListener};
	use crate::udp::protocol::frame::applications::ApplicationCommand;
	use crate::udp::protocol::frame::Frame;
	use crate::udp::protocol::frame::headers::Header;
	use crate::udp::protocol::reliable::ask::AskSender;
	use crate::udp::protocol::reliable::ask::header::AskFrameHeader;
	
	#[test]
	///
	/// Если не было входящих пакетов - то и не должно быть исходящих ask пакетов
	///
	fn should_ask_not_need_send() {
		let reliable = AskSender::default();
		assert_eq!(reliable.contains_self_data(&Instant::now()), false);
	}
	
	
	///
	/// На каждый входящий пакет должна быть ASK команда через определенное время
	///
	#[test]
	fn should_ask_need_send() {
		let mut reliable = AskSender::default();
		let time = Instant::now();
		let mut frame = Frame::new(10);
		frame.commands.reliability.push(ApplicationCommand::Ping("".to_string()));
		reliable.on_frame_received(&frame, &time);
		assert_eq!(reliable.contains_self_data(&time), false);
		assert_eq!(reliable.contains_self_data(&time.add(AskSender::SCHEDULE_SEND_TIME)), true);
	}
	
	///
	/// Проверяем формирование ASK заголовка на входящий пакет
	///
	#[test]
	fn should_send_ask_header() {
		let mut reliable = AskSender::default();
		let mut time = Instant::now();
		
		let mut in_frame = Frame::new(10);
		in_frame.commands.reliability.push(ApplicationCommand::Ping("ping".to_string()));
		reliable.on_frame_received(&in_frame, &time);
		
		time = time.add(AskSender::SCHEDULE_SEND_TIME);
		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);
		
		let header = out_frame.headers.first(Header::predicate_AskFrame);
		assert!(matches!(header, Option::Some(v) if v.start_frame_id == in_frame.header.frame_id));
	}
	
	
	///
	/// ASK заголовок должен содержать в себе не только подтверждение последнего пакета
	/// Но и подтверждение предыдущих, для более надежной доставки ASK заголовков
	///
	#[test]
	fn should_send_ask_header_for_prev_frames() {
		let mut reliable = AskSender::default();
		let mut time = Instant::now();
		
		for i in 0..AskSender::BUFFER_SIZE {
			let mut in_frame = Frame::new(10 + i as u64);
			in_frame.commands.reliability.push(ApplicationCommand::Ping("ping".to_string()));
			reliable.on_frame_received(&in_frame, &time);
		}
		
		time = time.add(AskSender::SCHEDULE_SEND_TIME);
		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);
		
		let header: Option<&AskFrameHeader> = out_frame.headers.first(Header::predicate_AskFrame);
		assert!(matches!(header, Option::Some(v) if v.start_frame_id == 10));
		let frames = header.unwrap().get_frames();
		
		for i in 0..frames.len() {
			assert_eq!(frames[i], 10 + i as u64)
		}
	}
	
	///
	/// Если входящие фреймы находятся далеко друга от друга (разница между frame_id)
	/// то должно сформироваться несколько ASK заголовков,
	/// так как каждый заголовок может содержать фреймы которые находятся близко друг от друга [AskFrameHeader]
	///
	#[test]
	fn should_ask_send_more_one_header() {
		let mut reliable = AskSender::default();
		let time = Instant::now();
		
		let mut frame_a = Frame::new(10);
		frame_a.commands.reliability.push(ApplicationCommand::Ping("".to_string()));
		reliable.on_frame_received(&frame_a, &time);
		
		let mut frame_b = Frame::new(10 + AskFrameHeader::CAPACITY as u64 + 1);
		frame_b.commands.reliability.push(ApplicationCommand::Ping("".to_string()));
		reliable.on_frame_received(&frame_b, &time);
		
		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);
		
		let headers: Vec<&AskFrameHeader> = out_frame.headers.find(Header::predicate_AskFrame);
		assert_eq!(headers.len(), 2);
		assert_eq!(headers[0].start_frame_id, frame_a.header.frame_id);
		assert_eq!(headers[1].start_frame_id, frame_b.header.frame_id);
	}
}
