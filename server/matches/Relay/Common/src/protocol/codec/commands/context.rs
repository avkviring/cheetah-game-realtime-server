use std::convert::TryFrom;
use std::fmt::format;
use std::io::{Cursor, ErrorKind};

use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error;

use crate::commands::CommandTypeId;
use crate::constants::FieldId;
use crate::protocol::codec::commands::header::CommandHeader;
use crate::protocol::codec::cursor::VariableInt;
use crate::protocol::frame::applications::ChannelGroup;
use crate::protocol::frame::codec::channel::ChannelTypeId;
use crate::room::object::GameObjectId;
use crate::room::owner::GameObjectOwner;
use crate::room::RoomMemberId;

///
/// Контекст команд - записывает/читает дельту изменений между командами, тем самым сокращаем
/// трафик, например при создании и загрузки данных объекта
///
#[derive(Debug, Default)]
pub struct CommandContext {
	object_id: Option<GameObjectId>,
	field_id: Option<FieldId>,
	channel_group_id: Option<ChannelGroup>,
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

#[derive(Debug, Error)]
pub enum CommandContextError {
	#[error("Context not contains object_id.")]
	ContextNotContainsObjectId,
	#[error("Context not contains creator.")]
	ContextNotContainsCreator,
	#[error("Context not contains channel group id.")]
	ContextNotContainsChannelGroupId,
	#[error("Context not contains field id.")]
	ContextNotContainsFieldId,
	#[error("IO error {:?}", .source)]
	Io {
		#[from]
		source: std::io::Error,
	},
}

impl CommandContext {
	pub(crate) fn get_creator(&self) -> Result<RoomMemberId, CommandContextError> {
		self.creator.ok_or(CommandContextError::ContextNotContainsCreator)
	}

	pub(crate) fn get_channel_group_id(&self) -> Result<ChannelGroup, CommandContextError> {
		self.channel_group_id
			.ok_or(CommandContextError::ContextNotContainsChannelGroupId)
	}

	pub(crate) fn get_field_id(&self) -> Result<ChannelGroup, CommandContextError> {
		self.field_id.ok_or(CommandContextError::ContextNotContainsCreator)
	}

	pub(crate) fn get_object_id(&self) -> Result<&GameObjectId, CommandContextError> {
		self.object_id.as_ref().ok_or(CommandContextError::ContextNotContainsObjectId)
	}

	///
	/// Записываем следующую порцию данных
	/// реальная запись будет осуществлена только если данные поменялись в сравнении с предыдущей
	/// командой
	///
	pub(crate) fn write_next(
		&mut self,
		object_id: Option<GameObjectId>,
		field_id: Option<FieldId>,
		channel_group: Option<ChannelGroup>,
		channel_type_id: ChannelTypeId,
		command_type_id: CommandTypeId,
		creator: Option<RoomMemberId>,
		out: &mut Cursor<&mut [u8]>,
	) -> std::io::Result<()> {
		let mut header = CommandHeader::new();
		header.command_type_id = command_type_id;
		header.channel_type_id = channel_type_id;

		let position = out.position();
		header.reserve(out);

		if compare_and_set(&mut self.object_id, object_id) {
			self.object_id.as_ref().unwrap().encode(out)?;
			header.new_object_id = true;
		}
		if compare_and_set(&mut self.field_id, field_id) {
			out.write_variable_u64(*self.field_id.as_ref().unwrap() as u64)?;
			header.new_field_id = true;
		}

		if compare_and_set(&mut self.channel_group_id, channel_group) {
			out.write_variable_u64(*self.channel_group_id.as_ref().unwrap() as u64)?;
			header.new_channel_group_id = true;
		}

		let creator_source = self.determinate_creator_source(creator);
		if let CreatorSource::New = creator_source {
			let creator = creator.unwrap();
			out.write_variable_u64(creator as u64)?;
			self.creator.replace(creator);
			header.creator_source = creator_source;
		}
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

		if let GameObjectOwner::User(user_id) = self.object_id.as_ref().unwrap().owner {
			if user_id == creator {
				return CreatorSource::AsObjectOwner;
			}
		}
		return CreatorSource::New;
	}

	///
	/// Читаем следующую порцию данных
	/// после чтения - контекст будет изменен на актуальные значения
	///
	pub(crate) fn read_next(&mut self, input: &mut Cursor<&mut [u8]>) -> Result<CommandHeader, CommandContextError> {
		let header = CommandHeader::decode(input)?;
		if header.new_object_id {
			self.object_id.replace(GameObjectId::decode(input)?);
		}
		if header.new_field_id {
			self.field_id.replace(input.read_variable_u64()? as FieldId);
		}
		if header.new_channel_group_id {
			self.channel_group_id.replace(input.read_variable_u64()? as ChannelGroup);
		}
		self.read_and_set_creator(input, &header.creator_source)?;
		Ok(header)
	}

	///
	/// Определяем автора команды, есть 4 способа его сохранения.
	/// Если успешно - сохраняем результат в контекст
	///
	fn read_and_set_creator(
		&mut self,
		input: &mut Cursor<&mut [u8]>,
		creator_source: &CreatorSource,
	) -> Result<Option<RoomMemberId>, CommandContextError> {
		match creator_source {
			CreatorSource::NotSupported => Ok(None),
			CreatorSource::Current => match &self.creator {
				None => Err(CommandContextError::ContextNotContainsCreator),
				Some(current) => Ok(Some(*current)),
			},
			CreatorSource::New => {
				let creator = input.read_variable_u64()? as RoomMemberId;
				self.creator.replace(creator);
				Ok(Some(creator))
			}
			CreatorSource::AsObjectOwner => match &self.object_id {
				None => Err(CommandContextError::ContextNotContainsObjectId),
				Some(object_id) => match &object_id.owner {
					GameObjectOwner::Room => Err(CommandContextError::ContextNotContainsObjectId),
					GameObjectOwner::User(user) => {
						self.creator.replace(*user);
						Ok(Some(*user))
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
			match context {
				None => true, // данные в контексте отсутствуют - записываем новые данные
				Some(value_context) if value_current == *value_context => false,
				_ => {
					context.replace(value_current);
					true
				}
			}
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
			_ => {
				return Err(std::io::Error::new(
					ErrorKind::InvalidData,
					format!("Invalid tag {} CreatorSource", value),
				))
			}
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
