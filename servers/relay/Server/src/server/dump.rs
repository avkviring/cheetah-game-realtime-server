use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::{GameObjectFields, HeapLessFloatMap, HeaplessBuffer, HeaplessLongMap};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::{RoomId, UserPublicKey};

use crate::room::object::GameObject;
use crate::room::{Room, User};
use crate::rooms::Rooms;
use crate::server::ServerThread;

///
/// Дамп внутреннего состояния сервера для отладки
///
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ServerDump {
	pub rooms: RoomsDump,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RoomsDump {
	pub room_by_id: HashMap<RoomId, RoomDump>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RoomDump {
	pub id: RoomId,
	pub users: Vec<UserDump>,
	pub objects: Vec<GameObjectDump>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UserDump {
	pub public_key: UserPublicKey,
	pub access_groups: AccessGroups,
	attached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameObjectDump {
	pub id: GameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFieldsDump,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameObjectFieldsDump {
	longs: HeaplessLongMap,
	floats: HeapLessFloatMap,
	structures: HashMap<FieldID, BinaryDump, FnvBuildHasher>,
}

impl From<&ServerThread> for ServerDump {
	fn from(server: &ServerThread) -> Self {
		Self {
			rooms: From::from(&server.rooms),
		}
	}
}

impl From<&Rooms> for RoomsDump {
	fn from(rooms: &Rooms) -> Self {
		let mut result = Self {
			room_by_id: Default::default(),
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
		let mut objects: Vec<GameObjectDump> = Default::default();
		room.objects.iter().for_each(|(_, o)| {
			objects.push(From::from(o));
		});

		let mut users = Vec::new();
		room.users.iter().for_each(|(_, user)| {
			users.push(From::from(user));
		});
		Self { id: room.id, users, objects }
	}
}

impl From<&GameObject> for GameObjectDump {
	fn from(source: &GameObject) -> Self {
		Self {
			id: source.id.clone(),
			template: source.template,
			access_groups: source.access_groups,
			fields: From::from(&source.fields),
		}
	}
}

impl From<&GameObjectFields> for GameObjectFieldsDump {
	fn from(fields: &GameObjectFields) -> Self {
		let fields = fields.clone();
		let mut structures: HashMap<FieldID, BinaryDump, FnvBuildHasher> = Default::default();
		fields.structures.iter().for_each(|(field, structure)| {
			structures.insert(*field, buffer_to_value(structure));
		});
		Self {
			longs: fields.longs,
			floats: fields.floats,
			structures,
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BinaryDump {
	MessagePack(rmpv::Value),
	Raw(HeaplessBuffer),
}

fn buffer_to_value(source: &HeaplessBuffer) -> BinaryDump {
	match rmpv::decode::value::read_value(&mut source.to_vec().as_slice()) {
		Ok(v) => BinaryDump::MessagePack(v),
		Err(_) => BinaryDump::Raw((*source).clone()),
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

impl ServerDump {
	pub fn to_json(&self) -> String {
		match serde_json::to_string_pretty(self) {
			Ok(v) => v,
			Err(e) => {
				panic!("{:?}", e);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use serde::{Deserialize, Serialize};

	use cheetah_relay_common::room::fields::HeaplessBuffer;
	use cheetah_relay_common::udp::bind_to_free_socket;

	use crate::room::object::GameObject;
	use crate::server::Server;

	#[derive(Serialize, Deserialize)]
	pub struct TestStruct {
		pub size: usize,
		pub x: u16,
	}

	#[test]
	fn should_dump() {
		let mut server = Server::new(bind_to_free_socket().unwrap().0, false);
		server.register_room(1).ok().unwrap();
		let mut object = GameObject {
			id: Default::default(),
			template: 0,
			access_groups: Default::default(),
			fields: Default::default(),
		};

		let mut data: HeaplessBuffer = Default::default();
		let msg_pack_data = rmp_serde::to_vec_named(&TestStruct { size: 100, x: 200 }).unwrap();
		for x in msg_pack_data {
			data.push(x).unwrap();
		}

		object.fields.structures.insert(1, data);

		server.create_object(1, object).unwrap();
		let result = server.dump();
		assert!(result.is_ok());

		let correct_result = r#"{
  "rooms": {
    "room_by_id": {
      "1": {
        "id": 1,
        "users": [],
        "objects": [
          {
            "id": {
              "owner": "Root",
              "id": 0
            },
            "template": 0,
            "access_groups": 0,
            "fields": {
              "longs": {},
              "floats": {},
              "structures": {
                "1": {
                  "MessagePack": {
                    "size": 100,
                    "x": 200
                  }
                }
              }
            }
          }
        ]
      }
    }
  }
}"#;

		let dump = result.unwrap().to_json();
		assert_eq!(dump, correct_result)
	}
}
