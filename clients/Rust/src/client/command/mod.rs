use cheetah_relay_common::network::command::{CommandCode, Decoder, Encoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::{UploadGameObjectC2SCommand, UploadGameObjectS2CCommand};
use cheetah_relay_common::network::niobuffer::{NioBuffer, NioBufferError};
use cheetah_relay_common::network::tcp::connection::OnReadBufferError;

pub mod upload;
pub mod long_counter;
pub mod float_counter;
pub mod structure;
pub mod event;
pub mod unload;

#[derive(Debug)]
pub enum C2SCommandUnion {
	Upload(UploadGameObjectC2SCommand),
	SetLongCounter(SetLongCounterCommand),
	IncrementLongCounter(IncrementLongCounterC2SCommand),
	SetFloatCounter(SetFloatCounterCommand),
	IncrementFloatCounter(IncrementFloatCounterC2SCommand),
	Structure(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
}

#[derive(Debug)]
pub enum S2CCommandUnion {
	Upload(UploadGameObjectS2CCommand),
	SetLongCounter(SetLongCounterCommand),
	SetFloatCounter(SetFloatCounterCommand),
	SetStruct(StructureCommand),
	Event(EventCommand),
	Unload(UnloadGameObjectCommand),
}


pub fn decode_command(read_buffer: &mut NioBuffer) -> Result<S2CCommandUnion, OnReadBufferError> {
	let command = read_buffer.read_u8().map_err(OnReadBufferError::NioBufferError)?;
	let result = match command {
		UploadGameObjectS2CCommand::COMMAND_CODE => {
			UploadGameObjectS2CCommand::decode(read_buffer).map(S2CCommandUnion::Upload)
		}
		EventCommand::COMMAND_CODE => {
			EventCommand::decode(read_buffer).map(S2CCommandUnion::Event)
		}
		StructureCommand::COMMAND_CODE => {
			StructureCommand::decode(read_buffer).map(S2CCommandUnion::SetStruct)
		}
		SetLongCounterCommand::COMMAND_CODE => {
			SetLongCounterCommand::decode(read_buffer).map(S2CCommandUnion::SetLongCounter)
		}
		SetFloatCounterCommand::COMMAND_CODE => {
			SetFloatCounterCommand::decode(read_buffer).map(S2CCommandUnion::SetFloatCounter)
		}
		UnloadGameObjectCommand::COMMAND_CODE => { UnloadGameObjectCommand::decode(read_buffer).map(S2CCommandUnion::Unload) }
		code => {
			return Result::Err(OnReadBufferError::UnknownCommand(code));
		}
	};
	result.map_err(OnReadBufferError::NioBufferError)
}

pub fn encode_command(buffer: &mut NioBuffer, command: &C2SCommandUnion) -> Result<(), NioBufferError> {
	match command {
		C2SCommandUnion::Upload(command) => {
			buffer.write_u8(UploadGameObjectC2SCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::SetLongCounter(command) => {
			buffer.write_u8(SetLongCounterCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::IncrementLongCounter(command) => {
			buffer.write_u8(IncrementLongCounterC2SCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::SetFloatCounter(command) => {
			buffer.write_u8(SetFloatCounterCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::IncrementFloatCounter(command) => {
			buffer.write_u8(IncrementFloatCounterC2SCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::Structure(command) => {
			buffer.write_u8(StructureCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::Event(command) => {
			buffer.write_u8(EventCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
		C2SCommandUnion::Unload(command) => {
			buffer.write_u8(UnloadGameObjectCommand::COMMAND_CODE)?;
			command.encode(buffer)
		}
	}
}