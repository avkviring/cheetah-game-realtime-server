use crate::commands::c2s::C2SCommand;
use crate::commands::field::FieldId;
use crate::commands::CommandDecodeError;
use crate::protocol::codec::commands::context::CommandContextError;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;
use byteorder::{ReadBytesExt, WriteBytesExt};
use num_traits::ToPrimitive;
use std::io::{Cursor, Error, ErrorKind};

#[derive(Debug, PartialEq, Clone)]
pub struct ForwardedCommand {
	pub creator: RoomMemberId,
	pub c2s: C2SCommand,
}

impl ForwardedCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.creator.into())?;
		out.write_u8(
			ToPrimitive::to_u8(&self.c2s.get_type_id())
				.ok_or_else(|| Error::new(ErrorKind::InvalidData, "could not write CommandTypeId to u8".to_owned()))?,
		)?;
		self.c2s.encode(out)
	}

	pub fn decode(
		object_id: Result<GameObjectId, CommandContextError>,
		field_id: Result<FieldId, CommandContextError>,
		input: &mut Cursor<&[u8]>,
	) -> Result<Self, CommandDecodeError> {
		let member_id = input
			.read_variable_u64()?
			.try_into()
			.map_err(|e| Error::new(ErrorKind::InvalidData, format!("could not convert member_id into RoomMemberId: {:?}", e)))?;
		let command_type_id = input.read_u8()?;
		let command_type_id = num::FromPrimitive::from_u8(command_type_id).ok_or(CommandContextError::UnknownCommandTypeId(command_type_id))?;
		Ok(ForwardedCommand {
			creator: member_id,
			c2s: C2SCommand::decode(command_type_id, object_id, field_id, input)?,
		})
	}
}
