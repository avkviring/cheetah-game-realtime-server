use std::sync::Mutex;

use cheetah_relay_common::commands::command::HeaplessBuffer;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserId;

use crate::controller::ClientController;
use crate::registry::Registry;

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
}

pub fn execute<F, T>(body: F) -> T
where
	F: FnOnce(&mut Registry) -> T,
{
	let mut clients = REGISTRY.lock().unwrap();
	let clients = &mut *clients;
	body(clients)
}

pub fn execute_with_client<F, T>(body: F) -> Result<T, ()>
where
	F: FnOnce(&mut ClientController) -> T,
{
	execute(|clients| match clients.current_client {
		None => {
			log::error!("current client not set");
			Result::Err(())
		}
		Some(ref client_id) => match clients.controllers.get_mut(client_id) {
			None => {
				log::error!("client not found {:?}", client_id);
				Result::Err(())
			}
			Some(client_api) => Result::Ok(body(client_api)),
		},
	})
}

#[repr(C)]
pub struct GameObjectIdFFI {
	id: u32,
	room_owner: bool,
	user_id: UserId,
}

impl GameObjectIdFFI {
	pub fn empty() -> Self {
		Self {
			id: 0,
			room_owner: false,
			user_id: 0,
		}
	}
}

impl From<&GameObjectId> for GameObjectIdFFI {
	fn from(from: &GameObjectId) -> Self {
		match from.owner {
			ObjectOwner::Root => GameObjectIdFFI {
				id: from.id,
				room_owner: true,
				user_id: UserId::max_value(),
			},
			ObjectOwner::User(user_id) => GameObjectIdFFI {
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
				owner: ObjectOwner::Root,
				id: from.id,
			},
			false => Self {
				owner: ObjectOwner::User(from.user_id),
				id: from.id,
			},
		}
	}
}

const BUFFER_MAX_SIZE: usize = 255;

#[repr(C)]
pub struct BufferFFI {
	pub len: u8,
	pub buffer: [u8; BUFFER_MAX_SIZE],
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
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::ffi::GameObjectIdFFI;

	#[test]
	fn should_convert_object_id() {
		let object_id = GameObjectId {
			owner: ObjectOwner::User(123),
			id: 100,
		};
		let object_id_fff = GameObjectIdFFI::from(&object_id);
		let converted_object_id = GameObjectId::from(&object_id_fff);
		assert_eq!(object_id, converted_object_id);
	}
}
