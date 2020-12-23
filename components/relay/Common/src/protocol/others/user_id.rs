use std::time::Instant;

use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;
use crate::protocol::FrameBuilder;
use crate::room::UserId;

///
/// Добавляет заголовок с UserPublicKey
/// Используется только на клиенте, для передачи ключа на сервер
///
#[derive(Debug)]
pub struct UserIdFrameBuilder(pub UserId);

impl FrameBuilder for UserIdFrameBuilder {
	fn contains_self_data(&self, _: &Instant) -> bool {
		false
	}
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		frame.headers.add(Header::UserId(self.0.clone()));
	}
}
