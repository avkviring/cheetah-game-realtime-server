use std::sync::Mutex;

use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::controller::ClientController;
use crate::registry::Clients;

pub mod logs;
pub mod command;
pub mod control;
pub mod channel;

lazy_static! {
    static ref CLIENTS: Mutex<Clients> = Mutex::new(Default::default());
}


pub fn execute<F, T>(body: F) -> T where F: FnOnce(&mut Clients) -> T
{
	let mut clients = CLIENTS.lock().unwrap();
	let clients = &mut *clients;
	body(clients)
}

pub fn execute_with_client<F, T>(body: F) -> Result<T, ()> where F: FnOnce(&mut ClientController) -> T
{
	execute(|clients| {
		match clients.current_client {
			None => {
				log::error!("current client not set");
				Result::Err(())
			}
			Some(ref client_id) => {
				match clients.controllers.get_mut(client_id) {
					None => {
						log::error!("client not found {:?}", client_id);
						Result::Err(())
					}
					Some(client_api) => {
						Result::Ok(body(client_api))
					}
				}
			}
		}
	})
}


#[repr(C)]
pub struct GameObjectIdFFI {
	id: u32,
	owner: UserPublicKey,
}

impl From<&GameObjectId> for GameObjectIdFFI {
	fn from(from: &GameObjectId) -> Self {
		let owner = if let ObjectOwner::User(public_key) = from.owner {
			public_key
		} else {
			0
		};
		Self {
			id: from.id,
			owner,
		}
	}
}

impl From<&GameObjectIdFFI> for GameObjectId {
	fn from(from: &GameObjectIdFFI) -> Self {
		Self {
			owner: ObjectOwner::User(from.owner),
			id: from.owner,
		}
	}
}


#[repr(C)]
pub struct BufferFFI {
	pub len: u8,
	pub buffer: *const u8,
}

impl From<&BufferFFI> for Vec<u8> {
	fn from(source: &BufferFFI) -> Self {
		unsafe {
			let slice = std::ptr::slice_from_raw_parts(source.buffer, source.len as usize);
			let mut result = Vec::new();
			result.copy_from_slice(&*slice);
			result
		}
	}
}

impl From<&Vec<u8>> for BufferFFI {
	fn from(source: &Vec<u8>) -> Self {
		BufferFFI {
			len: source.len() as u8,
			buffer: source.as_ptr(),
		}
	}
}