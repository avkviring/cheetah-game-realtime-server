use crate::commands::types::structure::BinaryField;
use crate::room::field::FieldId;
use crate::room::object::GameObjectId;
use cheetah_game_realtime_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use cheetah_game_realtime_protocol::RoomMemberId;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Error, ErrorKind};

///
/// Событие по объекту для определенного пользователя
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct TargetEvent {
	pub target: RoomMemberId,
	pub event: BinaryField,
}

impl TargetEvent {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(u64::from(self.target))?;
		self.event.encode(out)
	}

	pub fn decode(object_id: GameObjectId, field_id: FieldId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let target = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

		Ok(Self {
			target,
			event: BinaryField::decode(object_id, field_id, input)?,
		})
	}
}
