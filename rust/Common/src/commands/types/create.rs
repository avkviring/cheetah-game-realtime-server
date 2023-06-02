use std::io::{Cursor, Error, ErrorKind};

use byteorder::{ReadBytesExt, WriteBytesExt};
use cheetah_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};

use crate::room::access::AccessGroups;
use crate::room::buffer::Buffer;
use crate::room::object::{GameObjectId, GameObjectTemplateId};

///
/// Создать игровой объект от имени клиента
/// S->C, C->S
///
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub struct CreateGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
}

///
/// Игровой объект создан
///
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct C2SCreatedGameObjectCommand {
	pub object_id: GameObjectId,
	///
	/// Если room_owner true - то объект меняет идентификатор на идентификатор с owner=Room
	///
	pub room_owner: bool,

	///
	/// Если задан - то в комнате может быть только один объект с таким идентификатором
	///
	singleton_key: Buffer,
}

///
/// Игровой объект загружен на клиента
///
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct GameObjectCreatedS2CCommand {
	pub object_id: GameObjectId,
}

impl CreateGameObjectCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(u64::from(self.template))?;
		out.write_variable_u64(self.access_groups.0)
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let template = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
		let access_groups = AccessGroups(input.read_variable_u64()?);
		Ok(Self { object_id, template, access_groups })
	}
}

impl C2SCreatedGameObjectCommand {
	pub fn new(object_id: GameObjectId, room_owner: bool, singleton_key: Option<Buffer>) -> Self {
		Self {
			object_id,
			room_owner,
			singleton_key: singleton_key.unwrap_or_default(),
		}
	}

	pub fn get_singleton_key(&self) -> Option<&Buffer> {
		if self.singleton_key.len == 0 {
			None
		} else {
			Some(&self.singleton_key)
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_u8(u8::from(self.room_owner))?;
		match self.get_singleton_key() {
			None => out.write_variable_u64(0),
			Some(buffer) => buffer.encode(out),
		}
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let room_owner = input.read_u8()? == 1;
		let singleton_key = Buffer::decode(input)?;
		Ok(Self { object_id, room_owner, singleton_key })
	}
}
