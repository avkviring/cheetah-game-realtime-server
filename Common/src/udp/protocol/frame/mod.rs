use serde::{Deserialize, Serialize};

use crate::udp::protocol::frame::applications::ApplicationCommands;
use crate::udp::protocol::frame::headers::Headers;

pub mod headers;
pub mod applications;


pub type FrameId = u64;

///
/// Структура для передачи через UDP
///
#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
	pub header: FrameHeader,
	pub headers: Headers,
	///
	/// Сжимаются и шифруются
	///
	pub commands: ApplicationCommands,
}

///
/// Заголовок UDP фрейма
/// - не сжимается
/// - не шифруется
/// - защищен aead
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FrameHeader {
	///
	/// Версия протокола
	///
	pub protocol_version: u8,
	
	///
	/// Уникальный возрастающий идентификатор фрейма
	/// - игнорируем уже принятый фрейм с таким же frame_id
	/// - используется как nonce в алгоритме шифрования
	/// - должен быть уникальным, даже если это повторно отсылаемый фрейм
	///
	pub frame_id: FrameId,
}


impl Frame {
	pub const PROTOCOL_VERSION: u8 = 0;
	pub fn new(frame_id: FrameId) -> Self {
		Self {
			header: FrameHeader { protocol_version: Frame::PROTOCOL_VERSION, frame_id },
			headers: Default::default(),
			commands: ApplicationCommands::default(),
		}
	}
}