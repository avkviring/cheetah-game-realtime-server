use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_traits::FromPrimitive;

use crate::commands::context::{CommandContextError, CreatorSource};
use crate::commands::guarantees::codec::ChannelType;
use crate::commands::CommandTypeId;

///
/// Заголовок команды
/// Все данные заголовка сохраняются в u16
///
#[derive(Debug, Copy, Clone)]
pub struct CommandHeader {
	pub(crate) new_object_id: bool,
	pub(crate) new_field_id: bool,
	pub(crate) new_channel_group_id: bool,
	pub(crate) creator_source: CreatorSource,
	pub(crate) channel_type_id: ChannelType,
	pub(crate) command_type_id: CommandTypeId,
}

const NEW_OBJECT_ID_BIT: u16 = 15;
const NEW_FIELD_ID_BIT: u16 = 14;
const NEW_CHANNEL_GROUP_ID_BIT: u16 = 13;

impl CommandHeader {
	pub(crate) fn new() -> Self {
		Self {
			new_object_id: false,
			new_field_id: false,
			new_channel_group_id: false,
			creator_source: CreatorSource::NotSupported,
			channel_type_id: ChannelType(0),
			command_type_id: CommandTypeId::CreatedGameObject,
		}
	}
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> Result<CommandHeader, CommandContextError> {
		let header = input.read_u16::<BigEndian>()?;
		let command_type_id = (header & 0b11_1111) as u8;
		Ok(Self {
			new_object_id: (header & 1 << NEW_OBJECT_ID_BIT) > 0,
			new_field_id: (header & 1 << NEW_FIELD_ID_BIT) > 0,
			new_channel_group_id: (header & 1 << NEW_CHANNEL_GROUP_ID_BIT) > 0,
			creator_source: CreatorSource::try_from(((header & 0b110_0000_0000) >> 9) as u8)?,
			channel_type_id: ChannelType(((header & 0b1_1100_0000) >> 6) as u8),
			command_type_id: FromPrimitive::from_u8(command_type_id).ok_or(CommandContextError::UnknownCommandTypeId(command_type_id))?,
		})
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		assert!(self.channel_type_id < ChannelType(8));
		let mut header: u16 = 0;
		header += self.command_type_id as u16;
		header += u16::from(self.channel_type_id.0) << 6;
		header += u16::from(u8::from(&self.creator_source)) << 9;
		header += if self.new_channel_group_id { 1 << NEW_CHANNEL_GROUP_ID_BIT } else { 0 };
		header += if self.new_field_id { 1 << NEW_FIELD_ID_BIT } else { 0 };
		header += if self.new_object_id { 1 << NEW_OBJECT_ID_BIT } else { 0 };
		out.write_u16::<BigEndian>(header)
	}
	pub(crate) fn reserve(out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_u16::<BigEndian>(0)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::context::{CommandHeader, CreatorSource};
	use crate::commands::guarantees::codec::ChannelType;
	use crate::commands::CommandTypeId;

	#[test]
	fn test() {
		check(CommandHeader {
			new_object_id: false,
			new_field_id: false,
			new_channel_group_id: false,
			creator_source: CreatorSource::NotSupported,
			channel_type_id: ChannelType(0),
			command_type_id: CommandTypeId::CreateGameObject,
		});
		check(CommandHeader {
			new_object_id: true,
			new_field_id: false,
			new_channel_group_id: false,
			creator_source: CreatorSource::New,
			channel_type_id: ChannelType(7),
			command_type_id: CommandTypeId::CreateGameObject,
		});
		check(CommandHeader {
			new_object_id: false,
			new_field_id: true,
			new_channel_group_id: false,
			creator_source: CreatorSource::Current,
			channel_type_id: ChannelType(5),
			command_type_id: CommandTypeId::SetStructure,
		});

		check(CommandHeader {
			new_object_id: false,
			new_field_id: false,
			new_channel_group_id: true,
			creator_source: CreatorSource::AsObjectOwner,
			channel_type_id: ChannelType(3),
			command_type_id: CommandTypeId::CreateGameObject,
		});

		check(CommandHeader {
			new_object_id: true,
			new_field_id: true,
			new_channel_group_id: true,
			creator_source: CreatorSource::NotSupported,
			channel_type_id: ChannelType(7),
			command_type_id: CommandTypeId::CreateGameObject,
		});
	}

	fn check(header: CommandHeader) {
		let mut buffer = [0_u8; 10];
		let mut cursor = Cursor::new(buffer.as_mut());
		header.encode(&mut cursor).unwrap();
		let write_position = cursor.position();

		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = CommandHeader::decode(&mut read_cursor).unwrap();

		assert_eq!(write_position, read_cursor.position());
		assert_eq!(actual.command_type_id, header.command_type_id, "command_type");
		assert_eq!(actual.channel_type_id, header.channel_type_id, "channel_type_id");
		assert_eq!(actual.creator_source, header.creator_source, "creator_source");
		assert_eq!(actual.new_channel_group_id, header.new_channel_group_id, "new_channel_group_id");
		assert_eq!(actual.new_field_id, header.new_field_id, "new_field_id");
		assert_eq!(actual.new_object_id, header.new_object_id, "new_object_id");
	}
}
