use std::time::Instant;

use crate::commands::hash::UserPublicKey;
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::FrameBuilder;

///
/// Добавляет заголовок с UserPublicKey
/// Используется только на клиенте, для передачи ключа на сервер
///
pub struct UserPublicKeyFrameBuilder(pub UserPublicKey);


impl FrameBuilder for UserPublicKeyFrameBuilder {
	fn contains_self_data(&self, _: &Instant) -> bool {
		false
	}
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		frame.headers.add(Header::UserPublicKey(self.0.clone()));
	}
}