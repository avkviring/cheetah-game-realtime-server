use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::{RoomId, RoomMemberId};
use hash32_derive::Hash32;
use std::io::Cursor;
use std::io::ErrorKind::InvalidData;

#[derive(Debug, Default, Copy, Clone, PartialEq, Hash, Eq, Hash32)]
pub struct MemberAndRoomId {
	pub member_id: RoomMemberId,
	pub room_id: RoomId,
}

impl MemberAndRoomId {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			member_id: input
				.read_variable_u64()?
				.try_into()
				.map_err(|_| std::io::Error::new(InvalidData, "user_id is too large".to_string()))?,
			room_id: input.read_variable_u64()?,
		})
	}
	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(u64::from(self.member_id))?;
		out.write_variable_u64(self.room_id)
	}
}
