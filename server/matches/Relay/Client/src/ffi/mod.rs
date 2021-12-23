use std::sync::mpsc::SendError;
use std::sync::Mutex;

use lazy_static::lazy_static;
use thiserror::Error;

use cheetah_matches_relay_common::commands::CommandBuffer;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::registry::{ClientId, Registry};
use crate::clients::ClientRequest;

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

#[derive(Error, Debug)]
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
		None => Result::Err(ClientError::ClientNotFound(client_id)),
		Some(client_api) => action(client_api),
	})
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct GameObjectIdFFI {
	id: u32,
	room_owner: bool,
	user_id: RoomMemberId,
}

impl From<&GameObjectId> for GameObjectIdFFI {
	fn from(from: &GameObjectId) -> Self {
		match from.owner {
			GameObjectOwner::Room => GameObjectIdFFI {
				id: from.id,
				room_owner: true,
				user_id: RoomMemberId::MAX,
			},
			GameObjectOwner::User(user_id) => GameObjectIdFFI {
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
				owner: GameObjectOwner::User(from.user_id),
				id: from.id,
			},
		}
	}
}

const BUFFER_MAX_SIZE: usize = 255;

#[repr(C)]
#[derive(Clone, Eq, PartialEq)]
pub struct BufferFFI {
	pub len: u8,
	pub buffer: [u8; BUFFER_MAX_SIZE],
}

impl Default for BufferFFI {
	fn default() -> Self {
		Self {
			len: 0,
			buffer: [0; BUFFER_MAX_SIZE],
		}
	}
}

impl From<Vec<u8>> for BufferFFI {
	fn from(source: Vec<u8>) -> Self {
		let mut buffer = BufferFFI::default();
		buffer.len = source.len() as u8;
		buffer.buffer[0..source.len()].copy_from_slice(source.as_slice());
		buffer
	}
}

impl From<&BufferFFI> for CommandBuffer {
	fn from(source: &BufferFFI) -> Self {
		CommandBuffer::from_slice(&source.buffer[0..source.len as usize]).unwrap()
	}
}

impl From<&CommandBuffer> for BufferFFI {
	fn from(source: &CommandBuffer) -> Self {
		let mut result = BufferFFI {
			len: source.len() as u8,
			buffer: [0; BUFFER_MAX_SIZE],
		};
		let buffer = &mut result.buffer[0..source.len()];
		buffer.copy_from_slice(source);
		result
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::ffi::GameObjectIdFFI;

	#[test]
	fn should_convert_object_id() {
		let object_id = GameObjectId {
			owner: GameObjectOwner::User(123),
			id: 100,
		};
		let object_id_fff = GameObjectIdFFI::from(&object_id);
		let converted_object_id = GameObjectId::from(&object_id_fff);
		assert_eq!(object_id, converted_object_id);
	}
}
