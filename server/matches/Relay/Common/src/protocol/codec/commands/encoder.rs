use std::collections::VecDeque;
use std::io::Cursor;

use crate::commands::CommandTypeId;
use crate::constants::FieldId;
use crate::protocol::codec::channel::ChannelType;
use crate::protocol::codec::commands::context::CommandContext;
use crate::protocol::codec::cursor::VariableInt;
use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use crate::protocol::frame::channel::Channel;
use crate::room::object::GameObjectId;
use crate::room::RoomMemberId;

pub fn encode_commands(commands: &VecDeque<CommandWithChannel>, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
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

fn get_channel_info(command: &CommandWithChannel) -> (ChannelType, Option<ChannelGroup>) {
	let channel = &command.channel;
	let group = match channel {
		Channel::ReliableUnordered => None,
		Channel::ReliableOrderedByObject => None,
		Channel::ReliableOrderedByGroup(group) => Some(*group),
		Channel::UnreliableUnordered => None,
		Channel::UnreliableOrderedByObject => None,
		Channel::UnreliableOrderedByGroup(group) => Some(*group),
		Channel::ReliableSequenceByObject(_) => None,
		Channel::ReliableSequenceByGroup(group, _) => Some(*group),
	};
	(channel.get_type(), group)
}
fn get_command_info(
	command: &CommandWithChannel,
) -> (Option<GameObjectId>, Option<FieldId>, CommandTypeId, Option<RoomMemberId>) {
	match &command.command {
		BothDirectionCommand::S2CWithCreator(command_with_creator) => (
			command_with_creator.command.get_object_id(),
			command_with_creator.command.get_field_id(),
			command_with_creator.command.get_type_id(),
			Some(command_with_creator.creator),
		),
		BothDirectionCommand::C2S(c2s_command) => (
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
	command.channel.encode(out)?;
	match &command.command {
		BothDirectionCommand::TestSimple(_) => Ok(()),
		BothDirectionCommand::TestObject(_, _) => Ok(()),
		BothDirectionCommand::S2CWithCreator(command) => command.command.encode(out),
		BothDirectionCommand::C2S(command) => command.encode(out),
	}
}
