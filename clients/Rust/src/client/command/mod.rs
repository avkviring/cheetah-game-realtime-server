use cheetah_relay_common::network::command::CommandCode;
use cheetah_relay_common::network::command::C2SCommandUnion;
use cheetah_relay_common::network::command::S2CCommandUnion;
use cheetah_relay_common::network::command::S2CCommandWithMeta;
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{SetFloat64CounterCommand};
use cheetah_relay_common::network::command::load::LoadGameObjectCommand;
use cheetah_relay_common::network::command::long_counter::{SetLongCounterCommand};
use cheetah_relay_common::network::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use crate::client::C2SCommandWithMeta;

pub mod load;
pub mod long_counter;
pub mod float_counter;
pub mod structure;
pub mod event;
pub mod unload;


// pub fn decode_command(read_buffer: &mut NioBuffer) -> Result<S2CCommandWithMeta, OnReadBufferError> {
// 	let meta = S2CMetaCommandInformation::decode(read_buffer).map_err(OnReadBufferError::NioBufferError)?;
// 	let result = match meta.command_code {
// 		LoadGameObjectCommand::COMMAND_CODE => {
// 			LoadGameObjectCommand::decode(read_buffer).map(S2CCommandUnion::Load)
// 		}
// 		EventCommand::COMMAND_CODE => {
// 			EventCommand::decode(read_buffer).map(S2CCommandUnion::Event)
// 		}
// 		StructureCommand::COMMAND_CODE => {
// 			StructureCommand::decode(read_buffer).map(S2CCommandUnion::SetStruct)
// 		}
// 		SetLongCounterCommand::COMMAND_CODE => {
// 			SetLongCounterCommand::decode(read_buffer).map(S2CCommandUnion::SetLongCounter)
// 		}
// 		SetFloat64CounterCommand::COMMAND_CODE => {
// 			SetFloat64CounterCommand::decode(read_buffer).map(S2CCommandUnion::SetFloatCounter)
// 		}
// 		UnloadGameObjectCommand::COMMAND_CODE => { UnloadGameObjectCommand::decode(read_buffer).map(S2CCommandUnion::Unload) }
// 		code => {
// 			return Result::Err(OnReadBufferError::UnknownCommand(code));
// 		}
// 	};
// 	match result {
// 		Ok(command) => {
// 			Result::Ok(S2CCommandWithMeta { meta, command })
// 		}
// 		Err(error) => {
// 			Result::Err(OnReadBufferError::NioBufferError(error))
// 		}
// 	}
// }
//
// pub fn encode_command(buffer: &mut NioBuffer, command: &C2SCommandWithMeta) -> Result<(), NioBufferError> {
// 	command.meta.encode(buffer)?;
// 	match &command.command {
// 		C2SCommandUnion::Load(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::SetLongCounter(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::IncrementLongCounter(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::SetFloatCounter(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::IncrementFloatCounter(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::Structure(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::Event(command) => {
// 			command.encode(buffer)
// 		}
// 		C2SCommandUnion::Unload(command) => {
// 			command.encode(buffer)
// 		}
// 	}
// }