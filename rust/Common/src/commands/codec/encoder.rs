use std::collections::VecDeque;
use std::io::Cursor;

use byteorder::WriteBytesExt;
use cheetah_game_realtime_protocol::RoomMemberId;


use crate::commands::context::CommandContext;
use crate::commands::guarantees::codec::ChannelType;
use crate::commands::guarantees::ChannelGroup;
use crate::commands::{BothDirectionCommand, CommandTypeId, CommandWithReliabilityGuarantees};
use crate::room::field::FieldId;
use crate::room::object::GameObjectId;

pub(crate) fn encode_commands(commands: &mut VecDeque<CommandWithReliabilityGuarantees>, packet: &mut [u8]) -> (usize, bool) {
	let mut context = CommandContext::default();
	let mut contains_reliability_command = false;
	let mut cursor = Cursor::new(packet);
	cursor.set_position(1);
	let mut command_count = 0;
	while let Some(command) = commands.pop_front() {
		if command_count > 254 {
			commands.push_front(command);
			break;
		}
		let position = cursor.position();
		match encode_command(&mut context, &command, &mut cursor) {
			Ok(_) => {}
			Err(_) => {
				cursor.set_position(position);
				commands.push_front(command);
				break;
			}
		}
		command_count += 1;
		contains_reliability_command = contains_reliability_command || command.reliability_guarantees.is_reliable();
	}
	let size = cursor.position() as usize;
	cursor.set_position(0);
	cursor.write_u8(command_count).unwrap();
	(size, contains_reliability_command)
}

fn encode_command(context: &mut CommandContext, command: &CommandWithReliabilityGuarantees, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
	let (object_id, field_id, command_type_id, creator) = get_command_info(command);
	let (channel_type_id, channel_group) = get_channel_info(command);
	context.write_next(object_id, field_id, channel_group, channel_type_id, command_type_id, creator, out)?;
	command.reliability_guarantees.encode(out)?;
	match &command.command {
		BothDirectionCommand::S2CWithCreator(command) => command.command.encode(out),
		BothDirectionCommand::C2S(command) => command.encode(out),
	}
}

fn get_channel_info(command: &CommandWithReliabilityGuarantees) -> (ChannelType, Option<ChannelGroup>) {
	let channel = &command.reliability_guarantees;
	let group = channel.get_channel_group_id();
	(channel.get_type(), group)
}

fn get_command_info(command: &CommandWithReliabilityGuarantees) -> (Option<GameObjectId>, Option<FieldId>, CommandTypeId, Option<RoomMemberId>) {
	match &command.command {
		BothDirectionCommand::S2CWithCreator(command_with_creator) => (
			command_with_creator.command.get_object_id(),
			command_with_creator.command.get_field_id(),
			command_with_creator.command.get_type_id(),
			Some(command_with_creator.creator),
		),
		BothDirectionCommand::C2S(c2s_command) => (c2s_command.get_object_id(), c2s_command.get_field_id(), c2s_command.get_type_id(), None),
	}
}
