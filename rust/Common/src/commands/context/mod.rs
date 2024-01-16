use std::io::Cursor;
use std::num::TryFromIntError;

use byteorder::{ReadBytesExt, WriteBytesExt};
use cheetah_game_realtime_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use thiserror::Error;

use crate::commands::context::header::CommandHeader;
use crate::commands::guarantees::codec::ChannelType;
use crate::commands::guarantees::ChannelGroup;
use crate::commands::CommandTypeId;
use crate::room::field::FieldId;
use crate::room::object::GameObjectId;

pub mod header;

///
/// Контекст команд - записывает/читает дельту изменений между командами, тем самым сокращаем
/// трафик, например при создании и загрузки данных объекта
///
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CommandContext {
	object_id: Option<GameObjectId>,
	field_id: Option<FieldId>,
	channel_group: Option<ChannelGroup>,
}

#[derive(Error, Debug)]
pub enum CommandContextError {
	#[error("Context not contains object_id.")]
	ContextNotContainsObjectId,
	#[error("Context not contains creator.")]
	ContextNotContainsCreator,
	#[error("Context not contains channel group id.")]
	ContextNotContainsChannelGroupId,
	#[error("Context not contains field id.")]
	ContextNotContainsFieldId,
	#[error("IO error {0}")]
	Io(#[from] std::io::Error),
	#[error("Unknown command type id {0}")]
	UnknownCommandTypeId(u8),
	#[error("InputValueIsTooLarge {0}")]
	InputValueIsTooLarge(#[from] TryFromIntError),
}

impl CommandContext {
	pub(crate) fn get_channel_group_id(&self) -> Result<ChannelGroup, CommandContextError> {
		self.channel_group.ok_or(CommandContextError::ContextNotContainsChannelGroupId)
	}

	pub(crate) fn get_field_id(&self) -> Result<FieldId, CommandContextError> {
		self.field_id.ok_or(CommandContextError::ContextNotContainsCreator)
	}

	pub(crate) fn get_object_id(&self) -> Result<GameObjectId, CommandContextError> {
		self.object_id.ok_or(CommandContextError::ContextNotContainsObjectId)
	}

	///
	/// Записываем следующую порцию данных
	/// реальная запись будет осуществлена только если данные поменялись в сравнении с предыдущей
	/// командой
	///
	#[allow(clippy::too_many_arguments)]
	#[allow(clippy::unwrap_in_result)]
	pub(crate) fn write_next(
		&mut self,
		object_id: Option<GameObjectId>,
		field_id: Option<FieldId>,
		channel_group: Option<ChannelGroup>,
		channel_type_id: ChannelType,
		command_type_id: CommandTypeId,
		out: &mut Cursor<&mut [u8]>,
	) -> std::io::Result<()> {
		let mut header = CommandHeader::new();
		header.command_type_id = command_type_id;
		header.channel_type_id = channel_type_id;

		let position = out.position();
		CommandHeader::reserve(out)?;

		if compare_and_set(&mut self.object_id, object_id) {
			self.object_id.as_ref().unwrap().encode(out)?;
			header.new_object_id = true;
		}
		if compare_and_set(&mut self.field_id, field_id) {
			out.write_variable_u64(u64::from(*self.field_id.as_ref().unwrap()))?;
			header.new_field_id = true;
		}

		if compare_and_set(&mut self.channel_group, channel_group) {
			out.write_u8(self.channel_group.as_ref().unwrap().0)?;
			header.new_channel_group_id = true;
		}

		let current_position = out.position();
		out.set_position(position);
		header.encode(out)?;
		out.set_position(current_position);
		Ok(())
	}

	///
	/// Читаем следующую порцию данных
	/// после чтения - контекст будет изменен на актуальные значения
	///
	pub(crate) fn read_next(&mut self, input: &mut Cursor<&[u8]>) -> Result<CommandHeader, CommandContextError> {
		let header = CommandHeader::decode(input)?;
		if header.new_object_id {
			self.object_id.replace(GameObjectId::decode(input)?);
		}
		if header.new_field_id {
			self.field_id.replace(input.read_variable_u64()?.try_into()?);
		}
		if header.new_channel_group_id {
			self.channel_group.replace(ChannelGroup(input.read_u8()?));
		}
		Ok(header)
	}
}

///
/// Есть ли необходимость в использовании новых данных, или можно использовать текущие
///
fn compare_and_set<T: PartialEq>(context: &mut Option<T>, current: Option<T>) -> bool {
	match current {
		None => false, // нет данных - значит и не надо их использовать
		Some(value_current) => {
			let result = match context {
				None => true, // данные в контексте отсутствуют - записываем новые данные
				Some(value_context) if value_current == *value_context => false,
				_ => true,
			};
			context.replace(value_current);
			result
		}
	}
}

#[cfg(test)]
pub mod tests {
	use crate::commands::context::CommandContext;
	use crate::commands::guarantees::codec::ChannelType;
	use crate::commands::guarantees::ChannelGroup;
	use crate::commands::CommandTypeId;
	use crate::room::field::FieldId;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;
	use std::io::Cursor;

	struct Params {
		object_id: Option<GameObjectId>,
		field_id: Option<FieldId>,
		channel_group: Option<ChannelGroup>,
		channel_type_id: ChannelType,
		command_type_id: CommandTypeId,
		size: u64,
	}

	///
	/// Проверяем последовательное переключение контекста
	///
	#[test]
	fn test() {
		let params = vec![
			Params {
				object_id: None,
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2, //flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2 + 2, //flags + object_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 3, // flags + field_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 3, // flags + channel_group
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 4, // flags + object_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(10),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 3, // flags + field_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(10),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				size: 2, // flags
			},
		];

		check(&params);
	}

	fn check(params: &[Params]) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		let mut write_context = CommandContext::default();

		for param in params {
			let delta_size = cursor.position();
			write_context
				.write_next(param.object_id, param.field_id, param.channel_group, param.channel_type_id, param.command_type_id, &mut cursor)
				.unwrap();
			assert_eq!(cursor.position() - delta_size, param.size, "size");
		}
		let write_position = cursor.position();

		let mut read_context = CommandContext::default();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		for param in params {
			let header = read_context.read_next(&mut read_cursor).unwrap();
			assert_eq!(read_context.object_id, param.object_id, "object_id");
			assert_eq!(read_context.field_id, param.field_id, "field_id");
			assert_eq!(read_context.channel_group, param.channel_group, "channel_group_id");
			assert_eq!(header.channel_type_id, param.channel_type_id, "channel_type_id");
			assert_eq!(header.command_type_id, param.command_type_id, "command_type_id");
		}

		assert_eq!(write_position, read_cursor.position());
	}
}
