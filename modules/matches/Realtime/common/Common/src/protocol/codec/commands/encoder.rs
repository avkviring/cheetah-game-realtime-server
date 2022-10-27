use std::io::Cursor;

use crate::commands::field::FieldId;
use crate::commands::CommandTypeId;
use crate::protocol::codec::channel::ChannelType;
use crate::protocol::codec::commands::context::CommandContext;
use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

pub fn encode_command(context: &mut CommandContext, command: &CommandWithChannel, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
	let (object_id, field_id, command_type_id, creator) = get_command_info(command);
	let (channel_type_id, channel_group) = get_channel_info(command);
	context.write_next(object_id, field_id, channel_group, channel_type_id, command_type_id, creator, out)?;
	command.channel.encode(out)?;
	match &command.both_direction_command {
		BothDirectionCommand::S2CWithCreator(command) => command.command.encode(out),
		BothDirectionCommand::C2S(command) => command.encode(out),
	}
}

fn get_channel_info(command: &CommandWithChannel) -> (ChannelType, Option<ChannelGroup>) {
	let channel = &command.channel;
	let group = channel.get_channel_group_id();
	(channel.get_type(), group)
}
fn get_command_info(command: &CommandWithChannel) -> (Option<GameObjectId>, Option<FieldId>, CommandTypeId, Option<RoomMemberId>) {
	match &command.both_direction_command {
		BothDirectionCommand::S2CWithCreator(command_with_creator) => (
			command_with_creator.command.get_object_id(),
			command_with_creator.command.get_field_id(),
			command_with_creator.command.get_type_id(),
			Some(command_with_creator.creator),
		),
		BothDirectionCommand::C2S(c2s_command) => (c2s_command.get_object_id(), c2s_command.get_field_id(), c2s_command.get_type_id(), None),
	}
}
