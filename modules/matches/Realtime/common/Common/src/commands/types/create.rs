use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::commands::binary_value::BinaryValue;
use crate::constants::GameObjectTemplateId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::access::AccessGroups;
use crate::room::object::GameObjectId;

///
/// Создать игровой объект от имени клиента
/// S->C, C->S
///
#[derive(Debug, PartialEq, Clone)]
pub struct CreateGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
}

///
/// Игровой объект создан
///
#[derive(Debug, PartialEq, Clone)]
pub struct C2SCreatedGameObjectCommand {
	pub object_id: GameObjectId,
	///
	/// Если room_owner true - то объект меняет идентификатор на идентификатор с owner=Room
	///
	pub room_owner: bool,

	///
	/// Если задан - то в комнате может быть только один объект с таким идентификатором
	///
	pub singleton_key: Option<BinaryValue>,
}

///
/// Игровой объект загружен на клиента
///  
#[derive(Debug, PartialEq, Clone)]
pub struct GameObjectCreatedS2CCommand {
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

impl C2SCreatedGameObjectCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_u8(if self.room_owner { 1 } else { 0 })?;
		match &self.singleton_key {
			None => out.write_variable_u64(0),
			Some(buffer) => buffer.encode(out),
		}
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let room_owner = input.read_u8()? == 1;
		let unique_key = BinaryValue::decode(input).map(|v| if v.len() != 0 { Some(v) } else { None })?;
		Ok(Self {
			object_id,
			room_owner,
			singleton_key: unique_key,
		})
	}
}
