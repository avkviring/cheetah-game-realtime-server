use std::sync::mpsc::SendError;
use std::sync::Mutex;

use lazy_static::lazy_static;
use thiserror::Error;

use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
use cheetah_matches_realtime_common::commands::{CommandTypeId, FieldType};
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::registry::{ClientId, Registry};
use crate::clients::ClientRequest;

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientError {
	#[error("Create client error {}",.0)]
	CreateClientError(String),
	#[error("Registry mutex error {}",.0)]
	RegistryMutex(String),
	#[error("Client not found {}",.0)]
	ClientNotFound(ClientId),
	#[error("Connection status mutex error {}",.0)]
	ConnectionStatusMutexError(String),
	#[error("Send task error {}",.source)]
	SendTaskError {
		#[from]
		source: SendError<ClientRequest>,
	},
}

impl ClientError {
	pub(crate) fn store_error_and_get_code(&self) -> u8 {
		let mut last_error = LAST_ERROR.lock().unwrap();
		let msg = format!("{:?}", self);
		*last_error = msg;

		match self {
			ClientError::RegistryMutex(_) => 1,
			ClientError::ClientNotFound(_) => 2,
			ClientError::ConnectionStatusMutexError { .. } => 3,
			ClientError::SendTaskError { .. } => 4,
			ClientError::CreateClientError(_) => 5,
		}
	}
}

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

pub fn execute<F, R>(body: F) -> u8
where
	F: FnOnce(&mut Registry) -> Result<R, ClientError>,
{
	let mut lock = REGISTRY.lock();
	match lock.as_mut() {
		Ok(registry) => match body(registry) {
			Ok(_) => 0,
			Err(e) => e.store_error_and_get_code(),
		},
		Err(e) => {
			let error = ClientError::RegistryMutex(format!("{:?}", e));
			error.store_error_and_get_code()
		}
	}
}

pub fn execute_with_client<F, R>(client_id: ClientId, action: F) -> u8
where
	F: FnOnce(&mut ApplicationThreadClient) -> Result<R, ClientError>,
{
	execute(|registry| match registry.clients.get_mut(&client_id) {
		None => Err(ClientError::ClientNotFound(client_id)),
		Some(client_api) => action(client_api),
	})
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardedCommandFFI {
	command_type_id: CommandTypeId,
	creator: RoomMemberId,
	game_object_id: GameObjectIdFFI,
	field_id: FieldId,
	has_field_type: bool,
	field_type: FieldTypeFFI,
}

impl From<&ForwardedCommand> for ForwardedCommandFFI {
	fn from(c: &ForwardedCommand) -> Self {
		ForwardedCommandFFI {
			command_type_id: c.c2s.get_type_id(),
			creator: c.creator,
			game_object_id: c.c2s.get_object_id().unwrap_or_default().into(),
			field_id: c.c2s.get_field_id().unwrap_or_default(),
			has_field_type: c.c2s.get_field_type().is_some(),
			field_type: c.c2s.get_field_type().unwrap_or(FieldType::Long).into(),
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObjectIdFFI {
	id: u32,
	pub room_owner: bool,
	user_id: RoomMemberId,
}

impl Default for GameObjectIdFFI {
	fn default() -> Self {
		GameObjectId::default().into()
	}
}

impl From<GameObjectId> for GameObjectIdFFI {
	fn from(source: GameObjectId) -> Self {
		(&source).into()
	}
}

impl From<&GameObjectId> for GameObjectIdFFI {
	fn from(from: &GameObjectId) -> Self {
		match from.owner {
			GameObjectOwner::Room => GameObjectIdFFI {
				id: from.id,
				room_owner: true,
				user_id: RoomMemberId::MAX,
			},
			GameObjectOwner::Member(user_id) => GameObjectIdFFI {
				id: from.id,
				room_owner: false,
				user_id,
			},
		}
	}
}

impl From<&GameObjectIdFFI> for GameObjectId {
	fn from(from: &GameObjectIdFFI) -> Self {
		match from.room_owner {
			true => Self {
				owner: GameObjectOwner::Room,
				id: from.id,
			},
			false => Self {
				owner: GameObjectOwner::Member(from.user_id),
				id: from.id,
			},
		}
	}
}

const BUFFER_MAX_SIZE: usize = 255;

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BufferFFI {
	pub len: u8,
	pub pos: u8, // используется в C#
	pub buffer: [u8; BUFFER_MAX_SIZE],
}

impl Default for BufferFFI {
	fn default() -> Self {
		Self {
			len: 0,
			buffer: [0; BUFFER_MAX_SIZE],
			pos: 0,
		}
	}
}

impl From<Vec<u8>> for BufferFFI {
	fn from(source: Vec<u8>) -> Self {
		let mut buffer = BufferFFI {
			len: source.len() as u8,
			..Default::default()
		};
		buffer.buffer[0..source.len()].copy_from_slice(source.as_slice());
		buffer
	}
}

impl From<&BufferFFI> for BinaryValue {
	fn from(source: &BufferFFI) -> Self {
		BinaryValue::from(&source.buffer[0..source.len as usize])
	}
}

impl From<&BinaryValue> for BufferFFI {
	fn from(source: &BinaryValue) -> Self {
		let mut result = BufferFFI {
			len: source.len() as u8,
			pos: 0,
			buffer: [0; BUFFER_MAX_SIZE],
		};
		let buffer = &mut result.buffer[0..source.len()];
		buffer.copy_from_slice(source.as_slice());
		result
	}
}

#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldTypeFFI {
	Long,
	Double,
	Structure,
	Event,
}

impl From<FieldType> for FieldTypeFFI {
	fn from(source: FieldType) -> Self {
		(&source).into()
	}
}

impl From<&FieldType> for FieldTypeFFI {
	fn from(source: &FieldType) -> Self {
		match source {
			FieldType::Long => FieldTypeFFI::Long,
			FieldType::Double => FieldTypeFFI::Double,
			FieldType::Structure => FieldTypeFFI::Structure,
			FieldType::Event => FieldTypeFFI::Event,
		}
	}
}

impl From<FieldTypeFFI> for FieldType {
	fn from(source: FieldTypeFFI) -> Self {
		match source {
			FieldTypeFFI::Long => FieldType::Long,
			FieldTypeFFI::Double => FieldType::Double,
			FieldTypeFFI::Structure => FieldType::Structure,
			FieldTypeFFI::Event => FieldType::Event,
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
	use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
	use cheetah_matches_realtime_common::commands::types::event::{EventCommand, TargetEventCommand};
	use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
	use cheetah_matches_realtime_common::commands::CommandTypeId;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;

	use crate::ffi::{FieldTypeFFI, ForwardedCommandFFI, GameObjectIdFFI};

	#[test]
	fn should_convert_object_id() {
		let object_id = GameObjectId {
			owner: GameObjectOwner::Member(123),
			id: 100,
		};
		let object_id_fff = GameObjectIdFFI::from(&object_id);
		let converted_object_id = GameObjectId::from(&object_id_fff);
		assert_eq!(object_id, converted_object_id);
	}

	#[test]
	fn should_convert_forwarded_to_ffi() {
		let command = ForwardedCommand {
			creator: 123,
			c2s: C2SCommand::AttachToRoom,
		};
		let ffi = ForwardedCommandFFI::from(&command);
		assert_eq!(
			ForwardedCommandFFI {
				command_type_id: CommandTypeId::AttachToRoom,
				creator: 123,
				game_object_id: Default::default(),
				field_id: Default::default(),
				has_field_type: false,
				field_type: FieldTypeFFI::Long,
			},
			ffi
		)
	}

	#[test]
	fn should_convert_forwarded_to_ffi_with_field() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		let command = ForwardedCommand {
			creator: 123,
			c2s: C2SCommand::TargetEvent(TargetEventCommand {
				target: 10,
				event: EventCommand {
					object_id: object_id.clone(),
					field_id,
					event: BinaryValue::from(vec![1, 2, 3, 4].as_slice()),
				},
			}),
		};
		let ffi = ForwardedCommandFFI::from(&command);
		assert_eq!(
			ForwardedCommandFFI {
				command_type_id: CommandTypeId::TargetEvent,
				creator: 123,
				game_object_id: object_id.into(),
				field_id,
				has_field_type: true,
				field_type: FieldTypeFFI::Event,
			},
			ffi
		)
	}
}
