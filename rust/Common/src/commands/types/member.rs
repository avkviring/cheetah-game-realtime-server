use std::io::{Cursor, Error, ErrorKind};

use cheetah_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use cheetah_protocol::RoomMemberId;
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct MemberConnected {
	pub member_id: RoomMemberId,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct MemberDisconnected {
	pub member_id: RoomMemberId,
}

impl MemberConnected {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.member_id.into())
	}

	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let member_id = input
			.read_variable_u64()?
			.try_into()
			.map_err(|e| Error::new(ErrorKind::InvalidData, format!("could not convert member_id into RoomMemberId: {e:?}")))?;
		Ok(Self { member_id })
	}
}

impl MemberDisconnected {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.member_id.into())
	}

	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let member_id = input
			.read_variable_u64()?
			.try_into()
			.map_err(|e| Error::new(ErrorKind::InvalidData, format!("could not convert member_id into RoomMemberId: {e:?}")))?;
		Ok(Self { member_id })
	}
}
