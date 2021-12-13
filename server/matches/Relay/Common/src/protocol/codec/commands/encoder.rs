use std::collections::VecDeque;
use std::io::Cursor;

use crate::commands::CommandTypeId;
use crate::constants::FieldId;
use crate::protocol::codec::commands::context::CommandContext;
use crate::protocol::codec::cursor::VariableInt;
use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, ChannelSequence, CommandWithChannel};
use crate::protocol::frame::channel::CommandChannel;
use crate::protocol::frame::codec::channel::ChannelTypeId;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

pub fn encode(commands: &VecDeque<CommandWithChannel>, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
	out.write_variable_u64(commands.len() as u64)?;
	let mut context = CommandContext::default();
	for command in commands {
		let (object_id, field_id, command_type_id, creator) = get_command_info(command);
		let (channel_type_id, channel_group) = get_channel_info(&command);
		context.write_next(
			object_id,
			field_id,
			channel_group,
			channel_type_id,
			command_type_id,
			creator,
			out,
		)?;
		encode_command(command, out)?;
	}
	Result::Ok(())
}

fn get_channel_info(command: &CommandWithChannel) -> (ChannelTypeId, Option<ChannelGroup>) {
	let channel = &command.channel;
	let group = match channel {
		CommandChannel::ReliableUnordered => None,
		CommandChannel::ReliableOrderedByObject => None,
		CommandChannel::ReliableOrderedByGroup(group) => Some(*group),
		CommandChannel::UnreliableUnordered => None,
		CommandChannel::UnreliableOrderedByObject => None,
		CommandChannel::UnreliableOrderedByGroup(group) => Some(*group),
		CommandChannel::ReliableSequenceByObject(_) => None,
		CommandChannel::ReliableSequenceByGroup(group, _) => Some(*group),
	};
	(channel.get_type_id(), group)
}
fn get_command_info(
	command: &CommandWithChannel,
) -> (Option<GameObjectId>, Option<FieldId>, CommandTypeId, Option<RoomMemberId>) {
	match &command.command {
		BothDirectionCommand::S2CCommandWithCreator(command_with_creator) => (
			command_with_creator.command.get_object_id(),
			command_with_creator.command.get_field_id(),
			command_with_creator.command.get_type_id(),
			Some(command_with_creator.creator),
		),
		BothDirectionCommand::C2SCommand(c2s_command) => (
			c2s_command.get_object_id(),
			c2s_command.get_field_id(),
			c2s_command.get_type_id(),
			None,
		),
		BothDirectionCommand::TestSimple(_) => {
			todo!()
		}
		BothDirectionCommand::TestObject(_, _) => {
			todo!()
		}
	}
}
fn encode_command(command: &CommandWithChannel, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
	match &command.command {
		BothDirectionCommand::TestSimple(_) => Ok(()),
		BothDirectionCommand::TestObject(_, _) => Ok(()),
		BothDirectionCommand::S2CCommandWithCreator(command) => command.command.encode(out),
		BothDirectionCommand::C2SCommand(command) => command.encode(out),
	}
}
