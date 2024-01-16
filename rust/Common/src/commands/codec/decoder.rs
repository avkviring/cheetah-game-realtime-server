use std::io::Cursor;

use byteorder::ReadBytesExt;
use thiserror::Error;

use crate::commands::c2s::C2SCommand;
use crate::commands::context::header::CommandHeader;
use crate::commands::context::{CommandContext, CommandContextError};
use crate::commands::guarantees::codec::CommandChannelDecodeError;
use crate::commands::guarantees::ReliabilityGuaranteesChannel;
use crate::commands::s2c::S2CCommand;
use crate::commands::{BothDirectionCommand, CommandDecodeError, CommandWithReliabilityGuarantees};

pub fn decode_commands(server_side: bool, body: &[u8]) -> Result<Vec<CommandWithReliabilityGuarantees>, CommandsDecoderError> {
	let mut commands: Vec<CommandWithReliabilityGuarantees> = Default::default();

	let mut input = Cursor::new(body);
	let length = input.read_u8()?;
	let mut context = CommandContext::default();
	for _ in 0..length {
		let header = context.read_next(&mut input)?;
		let command = decode_command(server_side, &mut input, &header, &context)?;
		commands.push(command);
	}
	Ok(commands)
}

fn decode_command(server_side: bool, input: &mut Cursor<&[u8]>, header: &CommandHeader, context: &CommandContext) -> Result<CommandWithReliabilityGuarantees, CommandsDecoderError> {
	Ok(CommandWithReliabilityGuarantees {
		reliability_guarantees: ReliabilityGuaranteesChannel::decode(&header.channel_type_id, context.get_channel_group_id(), input)?,
		command: if server_side {
			BothDirectionCommand::C2S(C2SCommand::decode(header.command_type_id, context.get_object_id(), context.get_field_id(), input)?)
		} else {
			BothDirectionCommand::S2C(S2CCommand::decode(&header.command_type_id, context.get_object_id(), context.get_field_id(), input)?)
		},
	})
}

#[derive(Error, Debug)]
pub enum CommandsDecoderError {
	#[error("IO error {0}")]
	Io(#[from] std::io::Error),
	#[error("ApplicationCommandChannel error {0}")]
	ApplicationCommandChannel(#[from] CommandChannelDecodeError),
	#[error("CommandDecodeError error {0}")]
	CommandDecode(#[from] CommandDecodeError),
	#[error("CommandContextError error {0}")]
	CommandContext(#[from] CommandContextError),
}
