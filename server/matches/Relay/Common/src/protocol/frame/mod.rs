use std::collections::VecDeque;

use crate::protocol::frame::applications::CommandWithChannel;
use crate::protocol::frame::headers::{Header, Headers};

pub mod applications;
pub mod channel;
pub mod headers;
pub type FrameId = u64;

///
/// Структура для передачи через UDP
///
#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
	///
	/// Уникальный возрастающий идентификатор фрейма
	/// - игнорируем уже принятый фрейм с таким же frame_id
	/// - используется как nonce в алгоритме шифрования
	/// - должен быть уникальным, даже если это повторно отсылаемый фрейм
	///
	pub frame_id: FrameId,

	pub headers: Headers,

	///
	/// С гарантией доставки
	///
	pub reliable: VecDeque<CommandWithChannel>,

	///
	/// Без гарантии доставки
	///
	pub unreliable: VecDeque<CommandWithChannel>,
}

impl Frame {
	pub const MAX_FRAME_SIZE: usize = 1024;
	pub const MAX_COMMAND_COUNT: usize = 64;

	pub fn new(frame_id: FrameId) -> Self {
		Self {
			frame_id,
			headers: Default::default(),
			reliable: Default::default(),
			unreliable: Default::default(),
		}
	}

	///
	///  Получить оригинальный frame_id
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	///
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_retransmit) {
			None => self.frame_id,
			Some(value) => value.original_frame_id,
		}
	}

	///
	/// Фрейм с надежной доставкой?
	///
	pub fn is_reliability(&self) -> bool {
		!self.reliable.is_empty()
	}
}
