use std::any::Any;
use std::convert::TryFrom;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::commands::CommandTypeId;
use crate::protocol::codec::commands::context::CreatorSource;
use crate::protocol::frame::codec::channel::ChannelType;

///
/// Заголовок команды
/// Все данные заголовка сохраняются в u16
///
#[derive(Debug)]
pub(crate) struct CommandHeader {
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
			command_type_id: CommandTypeId(0),
		}
	}
	pub(crate) fn decode(input: &mut Cursor<&mut [u8]>) -> std::io::Result<CommandHeader> {
		let header = input.read_u16::<BigEndian>()?;
		Ok(Self {
			new_object_id: (header & 1 << NEW_OBJECT_ID_BIT) > 0,
			new_field_id: (header & 1 << NEW_FIELD_ID_BIT) > 0,
			new_channel_group_id: (header & 1 << NEW_CHANNEL_GROUP_ID_BIT) > 0,
			creator_source: CreatorSource::try_from(((header & 0b11000000000) >> 9) as u8)?,
			channel_type_id: ChannelType(((header & 0b111000000) >> 6) as u8),
			command_type_id: CommandTypeId((header & 0b111111) as u8),
		})
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		assert!(self.command_type_id.0 < 64);
		assert!(self.channel_type_id.0 < 8);
		let mut header: u16 = 0;
		header += self.command_type_id.0 as u16;
		header += (self.channel_type_id.0 as u16) << 6;
		header += (u8::from(&self.creator_source) as u16) << 9;
		header += if self.new_channel_group_id {
			1 << NEW_CHANNEL_GROUP_ID_BIT
		} else {
			0
		};
		header += if self.new_field_id { 1 << NEW_FIELD_ID_BIT } else { 0 };
		header += if self.new_object_id { 1 << NEW_OBJECT_ID_BIT } else { 0 };
		out.write_u16::<BigEndian>(header)
	}
	pub(crate) fn reserve(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_u16::<BigEndian>(0)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::CommandTypeId;
	use crate::protocol::codec::commands::context::CreatorSource;
	use crate::protocol::codec::commands::header::CommandHeader;
	use crate::protocol::frame::codec::channel::ChannelType;

	#[test]
	fn test() {
		check(CommandHeader {
			new_object_id: false,
			new_field_id: false,
			new_channel_group_id: false,
			creator_source: CreatorSource::NotSupported,
			channel_type_id: ChannelType(0),
			command_type_id: CommandTypeId(0),
		});
		check(CommandHeader {
			new_object_id: true,
			new_field_id: false,
			new_channel_group_id: false,
			creator_source: CreatorSource::New,
			channel_type_id: ChannelType(7),
			command_type_id: CommandTypeId(0),
		});
		check(CommandHeader {
			new_object_id: false,
			new_field_id: true,
			new_channel_group_id: false,
			creator_source: CreatorSource::Current,
			channel_type_id: ChannelType(5),
			command_type_id: CommandTypeId(7),
		});

		check(CommandHeader {
			new_object_id: false,
			new_field_id: false,
			new_channel_group_id: true,
			creator_source: CreatorSource::AsObjectOwner,
			channel_type_id: ChannelType(3),
			command_type_id: CommandTypeId(31),
		});

		check(CommandHeader {
			new_object_id: true,
			new_field_id: true,
			new_channel_group_id: true,
			creator_source: CreatorSource::NotSupported,
			channel_type_id: ChannelType(7),
			command_type_id: CommandTypeId(63),
		});
	}

	fn check(header: CommandHeader) {
		let mut buffer = [0 as u8; 10];
		let mut cursor = Cursor::new(buffer.as_mut());
		header.encode(&mut cursor).unwrap();
		cursor.set_position(0);
		let actual = CommandHeader::decode(&mut cursor).unwrap();
		assert_eq!(actual.command_type_id, header.command_type_id, "command_type");
		assert_eq!(actual.channel_type_id, header.channel_type_id, "channel_type_id");
		assert_eq!(actual.creator_source, header.creator_source, "creator_source");
		assert_eq!(
			actual.new_channel_group_id, header.new_channel_group_id,
			"new_channel_group_id"
		);
		assert_eq!(actual.new_field_id, header.new_field_id, "new_field_id");
		assert_eq!(actual.new_object_id, header.new_object_id, "new_object_id");
	}
}
