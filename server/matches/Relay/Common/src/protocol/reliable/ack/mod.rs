use crate::protocol::frame::headers::Header;
use crate::protocol::frame::{Frame, FrameId};
use crate::protocol::reliable::ack::header::AckHeader;
use crate::protocol::NOT_EXIST_FRAME_ID;

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

impl AckSender {
	pub fn contains_self_data(&self) -> bool {
		self.send_ack_counter > 0
	}

	pub fn build_frame(&mut self, frame: &mut Frame) {
		if self.send_ack_counter > 0 {
			self.send_ack_counter -= 1;
		}
		let mut frames = self.frames;
		frames.sort_unstable();

		let mut current_header: Option<AckHeader> = Option::None;
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
					let header = AckHeader::new(frame_id);
					current_header = Option::Some(header);
				}
				Some(ref mut header) => {
					if !header.store_frame_id(frame_id) {
						frame.headers.add(Header::Ack(header.clone()));
						let header = AckHeader::new(frame_id);
						current_header = Option::Some(header);
					}
				}
			}
		}

		match current_header {
			None => {}
			Some(header) => {
				frame.headers.add(Header::Ack(header));
			}
		}
	}

	pub fn on_frame_received(&mut self, frame: &Frame) {
		if frame.is_reliability() {
			self.send_ack_counter = AckSender::SEND_ACK_COUNTER;
			let mut frame_id = frame.frame_id;
			match frame.headers.first(Header::predicate_retransmit) {
				None => {}
				Some(header) => {
					frame_id = header.original_frame_id;
				}
			}

			// записываем frame_id в буфер
			// так как буфер кольцевой - то мы перепишем старый frame_id
			// если подтверждение для старого frame_id отправлялось недостаточное количество раз - то отметим данный факт
			let frame_position = self.next_frame_position;
			if self.ack_counts[frame_position] < AckSender::ALERT_LOW_COUNT_ACK
				&& self.frames[frame_position] != NOT_EXIST_FRAME_ID
			{
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

	use crate::commands::c2s::C2SCommand;
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::headers::{Header, HeaderVec};
	use crate::protocol::frame::Frame;
	use crate::protocol::reliable::ack::header::AckHeader;
	use crate::protocol::reliable::ack::AckSender;

	#[test]
	///
	/// Если не было входящих пакетов - то и не должно быть исходящих ack пакетов
	///
	fn should_ack_not_need_send() {
		let reliable = AckSender::default();
		assert!(!reliable.contains_self_data());
	}

	///
	/// Проверяем переключение внутреннего буфера
	///
	#[test]
	fn should_work_when_frame_count_more_buffer_size() {
		let mut reliable = AckSender::default();

		for i in 0..AckSender::BUFFER_SIZE + 10 {
			let mut frame = Frame::new(i as u64);
			frame.commands.push(create_command()).unwrap();
			reliable.on_frame_received(&frame);
		}
	}

	///
	/// На каждый входящий пакет должна быть ASK команда через определенное время
	///
	#[test]
	fn should_ack_need_send() {
		let mut reliable = AckSender::default();
		let mut frame = Frame::new(10);
		frame.commands.push(create_command()).unwrap();
		reliable.on_frame_received(&frame);
		assert!(reliable.contains_self_data());
	}

	///
	/// Проверяем формирование ACK заголовка на входящий пакет
	///
	#[test]
	fn should_send_ack_header() {
		let mut reliable = AckSender::default();

		let mut in_frame = Frame::new(10);
		in_frame.commands.push(create_command()).unwrap();
		reliable.on_frame_received(&in_frame);

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame);

		let header = out_frame.headers.first(Header::predicate_ack);
		assert!(matches!(header, Option::Some(v) if v.start_frame_id == in_frame.frame_id));
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
			in_frame.commands.push(create_command()).unwrap();
			reliable.on_frame_received(&in_frame);
		}

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame);

		let header: Option<&AckHeader> = out_frame.headers.first(Header::predicate_ack);
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
		frame_a
			.commands
			.push(CommandWithChannel {
				channel: Channel::ReliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			})
			.unwrap();
		reliable.on_frame_received(&frame_a);

		let mut frame_b = Frame::new(10 + AckHeader::CAPACITY as u64 + 1);
		frame_b.commands.push(create_command()).unwrap();
		reliable.on_frame_received(&frame_b);

		let mut out_frame = Frame::new(20);
		reliable.build_frame(&mut out_frame);

		let headers: HeaderVec<&AckHeader> = out_frame.headers.find(Header::predicate_ack);
		assert_eq!(headers.len(), 2);
		assert_eq!(headers[0].start_frame_id, frame_a.frame_id);
		assert_eq!(headers[1].start_frame_id, frame_b.frame_id);
	}

	fn create_command() -> CommandWithChannel {
		CommandWithChannel {
			channel: Channel::ReliableUnordered,
			command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		}
	}
}
