use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;
use crate::protocol::FrameBuilder;
use crate::room::{RoomId, RoomMemberId};

///
/// Добавляет заголовок с UserPublicKey
/// Используется только на клиенте, для передачи ключа на сервер
///
#[derive(Debug)]
pub struct MemberIdFrameBuilder(pub MemberAndRoomId);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct MemberAndRoomId {
	pub user_id: RoomMemberId,
	pub room_id: RoomId,
}

impl FrameBuilder for MemberIdFrameBuilder {
	fn contains_self_data(&self, _: &Instant) -> bool {
		false
	}
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		frame.headers.add(Header::UserAndRoomId(self.0.clone()));
	}
}
