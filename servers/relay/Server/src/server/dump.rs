use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::object::GameObject;
use crate::room::{Room, RoomId, User};
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
	longs: HashMap<FieldID, i64, FnvBuildHasher>,
	floats: HashMap<FieldID, f64, FnvBuildHasher>,
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
	Raw(Vec<u8>),
}

fn buffer_to_value(source: &Vec<u8>) -> BinaryDump {
	match rmpv::decode::value::read_value(&mut source.to_vec().as_slice()) {
		Ok(v) => BinaryDump::MessagePack(v),
		Err(_) => BinaryDump::Raw((*source).clone()),
	}
}

impl From<&User> for UserDump {
	fn from(user: &User) -> Self {
		Self {
			public_key: user.template.public_key,
			access_groups: user.template.access_groups,
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

	use cheetah_relay_common::udp::bind_to_free_socket;

	use crate::room::template::{GameObjectFieldsTemplate, GameObjectTemplate, RoomTemplate};
	use crate::server::Server;

	#[derive(Serialize, Deserialize)]
	pub struct TestStruct {
		pub size: usize,
		pub x: u16,
	}

	#[test]
	fn should_dump() {
		let mut server = Server::new(bind_to_free_socket().unwrap().0);
		let mut object_template = GameObjectTemplate {
			id: 0,
			template: 0,
			access_groups: Default::default(),
			fields: GameObjectFieldsTemplate {
				longs: Option::Some(Default::default()),
				floats: Option::Some(Default::default()),
				structures: Option::Some(Default::default()),
			},
		};

		object_template.fields.structures.as_mut().unwrap().insert(
			1,
			rmpv::Value::Map(vec![(
				rmpv::Value::String(rmpv::Utf8String::from("x")),
				rmpv::Value::Integer(rmpv::Integer::from(200)),
			)]),
		);

		server
			.register_room(RoomTemplate {
				id: 1,
				auto_create_user: false,
				users: vec![],
				objects: Option::Some(vec![object_template]),
			})
			.ok()
			.unwrap();

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
