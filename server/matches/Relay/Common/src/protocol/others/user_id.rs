use std::io::Cursor;

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::{RoomId, RoomMemberId};

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
