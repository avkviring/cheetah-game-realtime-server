use serde::{Deserialize, Serialize};

use crate::protocol::frame::applications::ApplicationCommands;
use crate::protocol::frame::headers::{Header, Headers};

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
	
	///
	///  Получить оригинальный frame_id
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	/// 
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_RetransmitFrame) {
			None => {
				self.header.frame_id
			}
			Some(value) => {
				value.original_frame_id
			}
		}
	}
	
	
	///
	/// Фрейм с надежной доставкой?
	/// 
	pub fn is_reliability(&self) -> bool {
		!self.commands.reliable.is_empty()
	}
}