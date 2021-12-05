use std::cell::RefCell;

use cheetah_matches_relay_common::commands::command::HeaplessBuffer;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::controller::ClientController;
use crate::registry::{ClientId, Registry};

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

thread_local! {
	static REGISTRY: RefCell<Registry> = RefCell::new(Default::default());
}

pub fn execute<F, T>(body: F) -> T
where
	F: FnOnce(&mut Registry) -> T,
{
	REGISTRY.with(|f| {
		let mut ref_mut = f.borrow_mut();
		body(&mut ref_mut)
	})
}

pub fn execute_with_client<F, R>(client_id: ClientId, action: F) -> Result<R, ()>
where
	F: FnOnce(&mut ClientController) -> R,
{
	execute(|registry| match registry.controllers.get_mut(&client_id) {
		None => {
			log::error!("client not found {:?}", client_id);
			Result::Err(())
		}
		Some(client_api) => {
			if !client_api.error_in_client_thread {
				Result::Ok(action(client_api))
			} else {
				registry.destroy_client(client_id);
				Result::Err(())
			}
		}
	})
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObjectIdFFI {
	id: u32,
	room_owner: bool,
	user_id: RoomMemberId,
}

impl GameObjectIdFFI {
	pub fn new() -> Self {
		Self {
			id: 0,
			room_owner: false,
			user_id: 0,
		}
	}
	pub fn empty() -> Self {
		Self {
			id: 0,
			room_owner: false,
			user_id: 0,
		}
	}

	pub fn stub() -> Self {
		Self {
			id: 5,
			room_owner: false,
			user_id: 77,
		}
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

impl BufferFFI {
	pub fn new() -> Self {
		Self {
			len: 0,
			buffer: [0; BUFFER_MAX_SIZE],
		}
	}

	pub fn stub() -> Self {
		Self {
			len: 3,
			buffer: [99; BUFFER_MAX_SIZE],
		}
	}
}

impl From<Vec<u8>> for BufferFFI {
	fn from(source: Vec<u8>) -> Self {
		let mut buffer = BufferFFI::new();
		buffer.len = source.len() as u8;
		buffer.buffer[0..source.len()].copy_from_slice(source.as_slice());
		buffer
	}
}

impl From<&BufferFFI> for HeaplessBuffer {
	fn from(source: &BufferFFI) -> Self {
		HeaplessBuffer::from_slice(&source.buffer[0..source.len as usize]).unwrap()
	}
}

impl From<&HeaplessBuffer> for BufferFFI {
	fn from(source: &HeaplessBuffer) -> Self {
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
