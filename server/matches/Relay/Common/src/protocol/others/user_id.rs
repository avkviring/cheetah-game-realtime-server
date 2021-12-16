use std::io::Cursor;
use std::time::Instant;

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::headers::Header;
use crate::protocol::frame::Frame;
use crate::protocol::FrameBuilder;
use crate::room::{RoomId, RoomMemberId};

///
/// Добавляет заголовок с UserPublicKey
/// Используется только на клиенте
///
#[derive(Debug)]
pub struct MemberIdFrameBuilder(pub MemberAndRoomId);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct MemberAndRoomId {
	pub user_id: RoomMemberId,
	pub room_id: RoomId,
}

impl MemberAndRoomId {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			user_id: input.read_variable_u64()? as u16,
			room_id: input.read_variable_u64()?,
		})
	}
	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.user_id as u64)?;
		out.write_variable_u64(self.room_id)
	}
}

impl FrameBuilder for MemberIdFrameBuilder {
	fn contains_self_data(&self, _: &Instant) -> bool {
		false
	}
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		frame.headers.add(Header::MemberAndRoomId(self.0.clone()));
	}
}
