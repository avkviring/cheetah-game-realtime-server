use crate::protocol::frame::headers::Header;
use crate::protocol::frame::{Frame, FrameId};
use crate::protocol::reliable::ack::header::AckFrameHeader;
use crate::protocol::{FrameBuilder, FrameReceivedListener, NOT_EXIST_FRAME_ID};
use std::time::Instant;

pub mod header;

///
/// Управление рассылкой подтверждения о приеме пакетов
/// - подтверждается [frame.header.frame_id], а не [frame.get_original_id()]
///
#[derive(Debug)]
pub struct AckSender {
	///
	/// Кольцевой буфер c frame_id,
	/// для всех фреймов из данного буфера производится отсылка ACK пакетов
	///
	pub frames: [FrameId; AckSender::BUFFER_SIZE],
	///
	/// Позиция для следующего фрейма
	///
	pub next_frame_position: usize,

	///
	/// Количество отправленных ACK пакетов для frame_id из [frames]
	/// Необходимо для сбора статистики и контроля достаточности количества отправленных ACK
	///
	pub ack_counts: [u8; AckSender::BUFFER_SIZE],

	///
	/// Время следующей отправки ACK
	///
	pub send_ack_counter: u8,

	///
	/// Количество фреймов для которых ACK отправлен недостаточное количество раз
	///
	pub low_count_ack_count: u64,
}

impl Default for AckSender {
	fn default() -> Self {
		Self {
			frames: [NOT_EXIST_FRAME_ID; AckSender::BUFFER_SIZE],
			next_frame_position: 0,
			ack_counts: [0; AckSender::BUFFER_SIZE],
			send_ack_counter: 0,
			low_count_ack_count: 0,
		}
	}
}

impl AckSender {
	///
	/// Минимальное количество отсылок подтверждения для одного пакета
	///
	pub const SEND_ACK_COUNTER: u8 = 3;

	///
	/// Количество подтверждаемых фреймов для одного исходящего пакета
	///
	pub const BUFFER_SIZE: usize = 64;

	pub const ALERT_LOW_COUNT_ACK: u8 = 2;
}

impl FrameBuilder for AckSender {
	fn contains_self_data(&self, _now: &Instant) -> bool {
		self.send_ack_counter > 0
	}

	fn build_frame(&mut self, frame: &mut Frame, _now: &Instant) {
		if self.send_ack_counter > 0 {
			self.send_ack_counter = self.send_ack_counter - 1;
		}
		let mut frames = self.frames.clone();
		frames.sort();

		let mut current_header: Option<AckFrameHeader> = Option::None;
		for i in 0..frames.len() {
			let frame_id = frames[i];
			if frame_id == NOT_EXIST_FRAME_ID {
				continue;
			}
			if self.ack_counts[i] < 254 {
				self.ack_counts[i] += 1;
			}
			match current_header {
				None => {
					let header = AckFrameHeader::new(frame_id);
					current_header = Option::Some(header);
				}
				Some(ref mut header) => {
					if !header.store_frame_id(frame_id) {
						frame.headers.add(Header::AckFrame(header.clone()));
						let header = AckFrameHeader::new(frame_id);
						current_header = Option::Some(header);
					}
				}
			}
		}

		match current_header {
			None => {}
			Some(header) => {
				frame.headers.add(Header::AckFrame(header));
			}
		}
	}
}

impl FrameReceivedListener for AckSender {
	fn on_frame_received(&mut self, frame: &Frame, _now: &Instant) {
		if frame.is_reliability() {
			self.send_ack_counter = AckSender::SEND_ACK_COUNTER;
			let mut frame_id = frame.header.frame_id;
			match frame.headers.first(Header::predicate_retransmit_frame) {
				None => {}
				Some(header) => {
					frame_id = header.original_frame_id;
				}
			}

			// записываем frame_id в буфер
			// так как буфер кольцевой - то мы перепишем старый frame_id
			// если подтверждение для старого frame_id отправлялось недостаточное количество раз - то отметим данный факт
			let frame_position = self.next_frame_position;
			if self.ack_counts[frame_position] < AckSender::ALERT_LOW_COUNT_ACK && self.frames[frame_position] != NOT_EXIST_FRAME_ID {
				self.low_count_ack_count += 1;
			}
			self.frames[frame_position] = frame_id;
			self.ack_counts[frame_position] = 0;

			self.next_frame_position += 1;
			if self.next_frame_position == AckSender::BUFFER_SIZE {
				self.next_frame_position = 0;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;

	use crate::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::Frame;
	use crate::protocol::reliable::ack::header::AckFrameHeader;
	use crate::protocol::reliable::ack::AckSender;
	use crate::protocol::{FrameBuilder, FrameReceivedListener};

	#[test]
	///
	/// Если не было входящих пакетов - то и не должно быть исходящих ack пакетов
	///
	fn should_ack_not_need_send() {
		let reliable = AckSender::default();
		assert_eq!(reliable.contains_self_data(&Instant::now()), false);
	}

	///
	/// Проверяем переключение внутреннего буфера
	///
	#[test]
	fn should_work_when_frame_count_more_buffer_size() {
		let mut reliable = AckSender::default();

		for i in 0..AckSender::BUFFER_SIZE + 10 {
			let time = Instant::now();
			let mut frame = Frame::new(i as u64);
			frame.commands.reliable.push(create_command());
			reliable.on_frame_received(&frame, &time);
		}
	}

	///
	/// На каждый входящий пакет должна быть ASK команда через определенное время
	///
	#[test]
	fn should_ack_need_send() {
		let mut reliable = AckSender::default();
		let time = Instant::now();
		let mut frame = Frame::new(10);
		frame.commands.reliable.push(create_command());
		reliable.on_frame_received(&frame, &time);
		assert_eq!(reliable.contains_self_data(&time), true);
	}

	///
	/// Проверяем формирование ACK заголовка на входящий пакет
	///
	#[test]
	fn should_send_ack_header() {
		let mut reliable = AckSender::default();
		let time = Instant::now();

		let mut in_frame = Frame::new(10);
		in_frame.commands.reliable.push(create_command());
		reliable.on_frame_received(&in_frame, &time);

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);

		let header = out_frame.headers.first(Header::predicate_ack_frame);
		assert!(matches!(header, Option::Some(v) if v.start_frame_id == in_frame.header.frame_id));
	}

	///
	/// ASK заголовок должен содержать в себе не только подтверждение последнего пакета
	/// Но и подтверждение предыдущих, для более надежной доставки ASK заголовков
	///
	#[test]
	fn should_send_ack_header_for_prev_frames() {
		let mut reliable = AckSender::default();
		let time = Instant::now();

		for i in 0..AckSender::BUFFER_SIZE {
			let mut in_frame = Frame::new(10 + i as u64);
			in_frame.commands.reliable.push(create_command());
			reliable.on_frame_received(&in_frame, &time);
		}

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);

		let header: Option<&AckFrameHeader> = out_frame.headers.first(Header::predicate_ack_frame);
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
	fn should_ack_send_more_one_header() {
		let mut reliable = AckSender::default();
		let time = Instant::now();

		let mut frame_a = Frame::new(10);
		frame_a.commands.reliable.push(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::TestSimple("".to_string()),
		});
		reliable.on_frame_received(&frame_a, &time);

		let mut frame_b = Frame::new(10 + AckFrameHeader::CAPACITY as u64 + 1);
		frame_b.commands.reliable.push(create_command());
		reliable.on_frame_received(&frame_b, &time);

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame, &time);

		let headers: Vec<&AckFrameHeader> = out_frame.headers.find(Header::predicate_ack_frame);
		assert_eq!(headers.len(), 2);
		assert_eq!(headers[0].start_frame_id, frame_a.header.frame_id);
		assert_eq!(headers[1].start_frame_id, frame_b.header.frame_id);
	}

	fn create_command() -> ApplicationCommandDescription {
		ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::TestSimple("".to_string()),
		}
	}
}
