use std::convert::TryFrom;
use std::io::{Cursor, ErrorKind};
use std::num::TryFromIntError;

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::commands::field::FieldId;
use crate::commands::CommandTypeId;
use crate::protocol::codec::channel::ChannelType;
use crate::protocol::codec::commands::header::CommandHeader;
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::applications::ChannelGroup;
use crate::room::object::GameObjectId;
use crate::room::owner::GameObjectOwner;
use crate::room::RoomMemberId;

///
/// Контекст команд - записывает/читает дельту изменений между командами, тем самым сокращаем
/// трафик, например при создании и загрузки данных объекта
///
#[derive(Debug, Default, Clone)]
pub struct CommandContext {
	object_id: Option<GameObjectId>,
	field_id: Option<FieldId>,
	channel_group: Option<ChannelGroup>,
	creator: Option<RoomMemberId>,
}

///
/// Источник владельца
///
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreatorSource {
	///
	/// Нет необходимости сохранять данные о создателе (например для команд с клиента)
	///
	NotSupported,
	///
	/// Текущий
	///
	Current,
	///
	/// Новый - требуется записать id пользователя, после этого он станет текущим для других команд
	///
	New,
	///
	/// Равен владельцу игрового объекта команды, после этого он станет текущим для других команд
	///
	AsObjectOwner,
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
	pub(crate) fn get_creator(&self) -> Result<RoomMemberId, CommandContextError> {
		self.creator.ok_or(CommandContextError::ContextNotContainsCreator)
	}

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
		creator: Option<RoomMemberId>,
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

		let creator_source = self.determinate_creator_source(creator);
		if let CreatorSource::New = creator_source {
			let creator = creator.unwrap();
			out.write_variable_u64(u64::from(creator))?;
		};
		if let Some(creator) = creator {
			self.creator.replace(creator);
		}
		header.creator_source = creator_source;

		let current_position = out.position();
		out.set_position(position);
		header.encode(out)?;
		out.set_position(current_position);
		Ok(())
	}

	///
	/// Сравниваем создателя с текущим и с владельцем объекта
	///
	fn determinate_creator_source(&self, creator: Option<RoomMemberId>) -> CreatorSource {
		if creator.is_none() {
			return CreatorSource::NotSupported;
		}

		let creator = creator.unwrap();

		match &self.creator {
			None => {}
			Some(current_creator) if *current_creator == creator => return CreatorSource::Current,
			_ => {}
		};

		if self.object_id.is_none() {
			return CreatorSource::New;
		}

		if let GameObjectOwner::Member(member_id) = self.object_id.as_ref().unwrap().get_owner() {
			if member_id == creator {
				return CreatorSource::AsObjectOwner;
			}
		}
		CreatorSource::New
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
		self.read_and_set_creator(input, header.creator_source)?;
		Ok(header)
	}

	///
	/// Определяем автора команды, есть 4 способа его сохранения.
	/// Если успешно - сохраняем результат в контекст
	///
	fn read_and_set_creator(
		&mut self,
		input: &mut Cursor<&[u8]>,
		creator_source: CreatorSource,
	) -> Result<Option<RoomMemberId>, CommandContextError> {
		match creator_source {
			CreatorSource::NotSupported => Ok(None),
			CreatorSource::Current => match &self.creator {
				None => Err(CommandContextError::ContextNotContainsCreator),
				Some(current) => Ok(Some(*current)),
			},
			CreatorSource::New => {
				let creator = input.read_variable_u64()?.try_into()?;
				self.creator.replace(creator);
				Ok(Some(creator))
			}
			CreatorSource::AsObjectOwner => match &self.object_id {
				None => Err(CommandContextError::ContextNotContainsObjectId),
				Some(object_id) => match &object_id.get_owner() {
					GameObjectOwner::Room => Err(CommandContextError::ContextNotContainsObjectId),
					GameObjectOwner::Member(member_id) => {
						self.creator.replace(*member_id);
						Ok(Some(*member_id))
					}
				},
			},
		}
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

impl TryFrom<u8> for CreatorSource {
	type Error = std::io::Error;
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(match value {
			0 => CreatorSource::NotSupported,
			1 => CreatorSource::Current,
			2 => CreatorSource::New,
			3 => CreatorSource::AsObjectOwner,
			_ => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Invalid tag {value} CreatorSource"))),
		})
	}
}

impl From<&CreatorSource> for u8 {
	fn from(source: &CreatorSource) -> Self {
		match source {
			CreatorSource::NotSupported => 0,
			CreatorSource::Current => 1,
			CreatorSource::New => 2,
			CreatorSource::AsObjectOwner => 3,
		}
	}
}

#[cfg(test)]
pub mod tests {
	use std::io::Cursor;

	use crate::commands::field::FieldId;
	use crate::commands::CommandTypeId;
	use crate::protocol::codec::channel::ChannelType;
	use crate::protocol::codec::commands::context::CommandContext;
	use crate::protocol::frame::applications::ChannelGroup;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;
	use crate::room::RoomMemberId;

	struct Params {
		object_id: Option<GameObjectId>,
		field_id: Option<FieldId>,
		channel_group: Option<ChannelGroup>,
		channel_type_id: ChannelType,
		command_type_id: CommandTypeId,
		creator: Option<RoomMemberId>,
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
				creator: None,
				size: 2, //flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 2 + 2, //flags + object_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 3, // flags + field_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 3, // flags + channel_group
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: None,
				size: 2, // flags
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Room)),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(7),
				size: 3, // flags+creator
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(5),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(7),
				size: 4, // flags + object_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(10),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(7),
				size: 3, // flags + field_id
			},
			Params {
				object_id: Some(GameObjectId::new(0, GameObjectOwner::Member(5))),
				field_id: Some(10),
				channel_group: Some(ChannelGroup(100)),
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(5),
				size: 2, // flags
			},
		];

		check(&params);
	}

	#[test]
	fn test_creator() {
		let params = vec![
			Params {
				object_id: None,
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(5),
				size: 2 + 1, //flags + сохранение идентификатора пользователя
			},
			Params {
				object_id: None,
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(5),
				size: 2, //flags + пользователь не поменялся
			},
			Params {
				object_id: Some(GameObjectId::new(100, GameObjectOwner::Member(7))),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(5),
				size: 2 + 2, //flags + игровой объект
			},
			Params {
				object_id: Some(GameObjectId::new(100, GameObjectOwner::Member(7))),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(7),
				size: 2, //flags + пользователь равен владельцу игрового объекта, отдельного
				         // сохранения не требуется
			},
			Params {
				object_id: Some(GameObjectId::new(100, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(7),
				size: 4, //flags + object_id
			},
			Params {
				object_id: Some(GameObjectId::new(100, GameObjectOwner::Room)),
				field_id: None,
				channel_group: None,
				channel_type_id: ChannelType(5),
				command_type_id: CommandTypeId::CreateGameObject,
				creator: Some(9),
				size: 3, //flags + member_id
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
				.write_next(
					param.object_id,
					param.field_id,
					param.channel_group,
					param.channel_type_id,
					param.command_type_id,
					param.creator,
					&mut cursor,
				)
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
			assert_eq!(read_context.creator, param.creator, "creator");
		}

		assert_eq!(write_position, read_cursor.position());
	}
}
