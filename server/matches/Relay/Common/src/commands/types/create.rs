use std::io::{Cursor, Error, ErrorKind, Read, Write};

use crate::commands::CommandBuffer;
use crate::constants::GameObjectTemplateId;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::room::access::AccessGroups;
use crate::room::object::GameObjectId;

///
/// Создать игровой объект от имени клиента
///
#[derive(Debug, PartialEq, Clone)]
pub struct C2SCreateMemberGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
}

///
/// Создать игровой объект от имени комнаты
///
#[derive(Debug, PartialEq, Clone)]
pub struct C2SCreateRoomGameObjectCommand {
	///
	/// Временный идентификатор объекта, используется только для загрузки данных объекта
	/// между командами C2SCreateRoomGameObjectCommand -> C2SCreatedRoomGameObjectCommand
	///
	pub temporary_object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
	///
	/// Может существовать только один объект с данным ключем в комнате, если объект с таким
	/// ключем уже есть - команда будет проигнорировано
	///
	pub unique_create_key: Option<CommandBuffer>,
}

///
/// Игровой объект создан
///
#[derive(Debug, PartialEq, Clone)]
pub struct C2SCreatedGameObjectCommand {
	pub object_id: GameObjectId,
}

///
/// Начало загрузки игрового объекта на клиента
///
#[derive(Debug, PartialEq, Clone)]
pub struct S2CLoadingGameObjectCommand {
	pub object_id: GameObjectId,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
}

///
/// Игровой объект загружен на клиента
///  
#[derive(Debug, PartialEq, Clone)]
pub struct S2CLoadedGameObjectCommand {
	pub object_id: GameObjectId,
}

impl C2SCreateMemberGameObjectCommand {
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

impl S2CLoadingGameObjectCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.template as u64)?;
		out.write_variable_u64(self.access_groups.0)
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let template = input.read_variable_u64()? as GameObjectTemplateId;
		let access_groups = AccessGroups(input.read_variable_u64()?);
		Ok(Self {
			object_id: object_id,
			template,
			access_groups,
		})
	}
}

impl C2SCreateRoomGameObjectCommand {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.template as u64)?;
		out.write_variable_u64(self.access_groups.0)?;
		match self.unique_create_key {
			None => {
				out.write_variable_u64(0)?;
			}
			Some(ref key) => out.write_all(key.as_slice())?,
		};
		Ok(())
	}

	pub fn decode(object_id: GameObjectId, input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let template = input.read_variable_u64()? as GameObjectTemplateId;
		let access_groups = AccessGroups(input.read_variable_u64()?);

		let size = input.read_variable_u64()? as usize;
		let unique_create_key = if size == 0 {
			None
		} else {
			let mut buffer = CommandBuffer::new();
			if size > buffer.capacity() {
				return Err(Error::new(
					ErrorKind::InvalidData,
					format!("Unique create key buffer size to big {}", size),
				));
			}
			unsafe {
				buffer.set_len(size);
			}
			input.read_exact(&mut buffer[0..size])?;
			Some(buffer)
		};

		Ok(Self {
			temporary_object_id: object_id,
			template,
			access_groups,
			unique_create_key,
		})
	}
}
