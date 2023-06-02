use crate::frame::headers::{Header, Headers};

pub mod disconnected_reason;
pub mod headers;
pub mod member_private_key;
///
/// Уникальный возрастающий идентификатор фрейма
/// - игнорируем уже принятый фрейм с таким же `frame_id`
/// - используется как nonce в алгоритме шифрования
/// - должен быть уникальным, даже если это повторно отсылаемый фрейм
///
pub type FrameId = u64;

///
/// Идентификатор подключения, если пришел фрейм с новым id подключения, то сервер удаляем предыдущий протокол для данного пользователя
///
pub type ConnectionId = u64;

pub const FRAME_BODY_CAPACITY: usize = 450;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frame {
	pub connection_id: ConnectionId,
	pub frame_id: FrameId,
	pub headers: Headers,
	pub body_size: usize,
	pub reliability: bool,
	pub body: [u8; FRAME_BODY_CAPACITY],
}

impl Frame {
	#[must_use]
	pub fn new(connection_id: ConnectionId, frame_id: FrameId) -> Self {
		Self {
			connection_id,
			frame_id,
			headers: Default::default(),
			body_size: 0,
			body: [0; FRAME_BODY_CAPACITY],
			reliability: false,
		}
	}

	#[cfg(test)]
	pub(crate) fn set_body(&mut self, body: &[u8]) {
		self.body_size = body.len();
		self.body[0..self.body_size].copy_from_slice(body);
	}

	///
	///  Получить оригинальный `frame_id`
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	///
	#[must_use]
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_retransmit) {
			None => self.frame_id,
			Some(value) => value.original_frame_id,
		}
	}

	pub fn get_body(&self) -> &[u8] {
		&self.body[0..self.body_size]
	}
}
