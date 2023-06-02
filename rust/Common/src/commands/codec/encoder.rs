use std::collections::VecDeque;
use std::io::Cursor;

use cheetah_protocol::frame::FRAME_BODY_CAPACITY;
use cheetah_protocol::RoomMemberId;

use crate::commands::context::CommandContext;
use crate::commands::guarantees::codec::ChannelType;
use crate::commands::guarantees::ChannelGroup;
use crate::commands::{BothDirectionCommand, CommandTypeId, CommandWithReliabilityGuarantees};
use crate::room::field::FieldId;
use crate::room::object::GameObjectId;

pub(crate) fn encode_commands(commands: &mut VecDeque<CommandWithReliabilityGuarantees>, buffer: &mut [u8; FRAME_BODY_CAPACITY]) -> (usize, bool) {
	let mut tmp_buffer = [0; FRAME_BODY_CAPACITY * 2];
	let mut context = CommandContext::default();
	let mut contains_reliability_command = false;
	let mut cursor = Cursor::new(tmp_buffer.as_mut_slice());
	cursor.set_position(1);
	let mut command_count = 0;
	while let Some(command) = commands.pop_front() {
		let position = cursor.position();
		encode_command(&mut context, &command, &mut cursor).unwrap();
		if cursor.position() >= FRAME_BODY_CAPACITY as u64 {
			cursor.set_position(position);
			commands.push_front(command);
			break;
		}
		command_count += 1;
		contains_reliability_command = contains_reliability_command || command.reliability_guarantees.is_reliable();
	}
	let size = cursor.position() as usize;
	tmp_buffer[0] = command_count;
	buffer[0..size].copy_from_slice(&tmp_buffer[0..size]);
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
