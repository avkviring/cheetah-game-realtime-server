use crate::frame::headers::{Header, Headers};
use crate::frame::segment::Segment;

pub mod disconnected_reason;
pub mod headers;
pub mod member_private_key;
pub mod packets_collector;
pub mod segment;
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Frame {
	pub connection_id: ConnectionId,
	pub frame_id: FrameId,
	pub headers: Headers,
	pub reliability: bool,
	pub segment: Segment,
}

impl Frame {
	#[must_use]
	pub fn new(connection_id: ConnectionId, frame_id: FrameId, reliability: bool, segment: Segment) -> Self {
		Self {
			connection_id,
			frame_id,
			headers: Default::default(),
			reliability,
			segment,
		}
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
}
