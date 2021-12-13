use std::io::Cursor;

use crate::commands::CommandTypeId;
use crate::protocol::codec::commands::context::CreatorSource;
use crate::protocol::frame::codec::channel::ChannelTypeId;

// (o, f, с, c, g, x, ch, ch, ch, ch, cmd, cmd, cmd, cmd, cmd, cmd)
#[derive(Debug, Default)]
pub(crate) struct CommandHeader {
	pub(crate) new_object_id: bool,
	pub(crate) new_field_id: bool,
	pub(crate) new_channel_group_id: bool,
}

impl CommandHeader {
	pub(crate) fn get_creator_source(&self) -> CreatorSource {
		todo!()
	}
}

impl CommandHeader {
	pub(crate) fn decode(p0: &mut Cursor<&mut [u8]>) -> std::io::Result<CommandHeader> {
		todo!()
	}

	pub(crate) fn encode(
		&self,
		creator_source: CreatorSource,
		command_type_id: CommandTypeId,
		channel_type_id: ChannelTypeId,
		out: &mut Cursor<&mut [u8]>,
	) -> std::io::Result<()> {
		todo!()
	}
	pub(crate) fn new(command_type_id: CommandTypeId) -> Self {
		todo!()
	}
	pub(crate) fn reserve(&self, p0: &mut Cursor<&mut [u8]>) {
		todo!()
	}
	pub(crate) fn get_channel_type_id(&self) -> ChannelTypeId {
		todo!()
	}

	pub(crate) fn get_command_type_id(&self) -> CommandTypeId {
		todo!()
	}
}
