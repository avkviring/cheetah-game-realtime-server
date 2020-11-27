use std::collections::HashMap;

use fnv::FnvBuildHasher;
use indexmap::map::IndexMap;

use cheetah_relay_common::room::{RoomId, UserPublicKey};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use serde::{Deserialize, Serialize};

use crate::room::{Room, User};
use crate::room::object::GameObject;
use crate::rooms::Rooms;
use crate::server::{Server, ServerThread};

///
/// Дамп внутреннего состояния сервера для отладки
///
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ServerDump {
	pub rooms: RoomsDump
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RoomsDump {
	pub room_by_id: HashMap<RoomId, RoomDump>
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RoomDump {
	pub id: RoomId,
	pub users: HashMap<UserPublicKey, UserDump, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserDump {
	pub public_key: UserPublicKey,
	pub access_groups: AccessGroups,
	attached: bool,
}

impl From<&ServerThread> for ServerDump {
	fn from(server: &ServerThread) -> Self {
		Self {
			rooms: From::from(&server.rooms)
		}
	}
}

impl From<&Rooms> for RoomsDump {
	fn from(rooms: &Rooms) -> Self {
		let mut result = Self {
			room_by_id: Default::default()
		};
		rooms.room_by_id.iter().for_each(|(id, room)| {
			let room = &*room.borrow();
			result.room_by_id.insert(*id, From::from(room));
		});
		result
	}
}


impl From<&Room> for RoomDump {
	fn from(room: &Room) -> Self {
		let mut result = Self {
			id: room.id,
			users: Default::default(),
			objects: room.objects.clone(),
		};
		room.users.iter().for_each(|(id, user)| {
			result.users.insert(*id, From::from(user));
		});
		
		result
	}
}

impl From<&User> for UserDump {
	fn from(user: &User) -> Self {
		Self {
			public_key: user.public_key,
			access_groups: user.access_groups,
			attached: user.attached,
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::udp::bind_to_free_socket;
	
	use crate::server::Server;
	
	#[test]
	fn should_dump() {
		let mut server = Server::new(bind_to_free_socket().unwrap().0, false);
		server.register_room(1);
		server.register_room(2);
		let result = server.dump();
		assert!(result.is_ok());
		let dump = result.unwrap();
		assert_eq!(dump.rooms.room_by_id.len(), 2);
	}
}