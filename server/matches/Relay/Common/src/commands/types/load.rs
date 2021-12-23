use std::io::Cursor;

use crate::constants::GameObjectTemplateId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::access::AccessGroups;
use crate::room::object::GameObjectId;

///
/// Игровой объект создается
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone)]
pub struct CreateGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
}

///
/// Игровой объект создан
/// - направления C->S, S->C
///
#[derive(Debug, PartialEq, Clone)]
pub struct CreatedGameObjectCommand {
	pub object_id: GameObjectId,
}

impl CreateGameObjectCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.template as u64)?;
		out.write_variable_u64(self.access_groups.0)
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let template = input.read_variable_u64()? as GameObjectTemplateId;
		let access_groups = AccessGroups(input.read_variable_u64()?);
		Ok(Self {
			object_id,
			template,
			access_groups,
		})
	}
}