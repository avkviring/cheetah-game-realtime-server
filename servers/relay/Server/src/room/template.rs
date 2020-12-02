use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::{GameObjectFields, HeapLessFloatMap, HeaplessBuffer, HeaplessLongMap};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::room::object::GameObject;
use crate::room::RoomId;

///
/// Шаблон для создания комнаты
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RoomTemplate {
	pub id: RoomId,
	pub users: Vec<UserTemplate>,
	pub objects: Option<Vec<GameObjectTemplate>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserTemplate {
	pub public_key: UserPublicKey,
	pub private_key: UserPrivateKey,
	pub access_groups: AccessGroups,
	pub objects: Option<Vec<GameObjectTemplate>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectTemplate {
	pub id: u32,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFieldsTemplate,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectFieldsTemplate {
	pub longs: Option<HashMap<FieldID, i64, FnvBuildHasher>>,
	pub floats: Option<HashMap<FieldID, f64, FnvBuildHasher>>,
	pub structures: Option<HashMap<FieldID, rmpv::Value, FnvBuildHasher>>,
}

impl GameObjectTemplate {
	pub fn to_root_game_object(&self) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, ObjectOwner::Root))
	}
	pub fn to_user_game_object(&self, user_public_key: UserPublicKey) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, ObjectOwner::User(user_public_key)))
	}
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		let mut longs = HeaplessLongMap::new();
		if let Some(ref self_longs) = self.fields.longs {
			self_longs.iter().for_each(|(k, v)| {
				longs.insert(k.clone(), *v).unwrap();
			});
		}

		let mut floats = HeapLessFloatMap::new();
		if let Some(ref self_floats) = self.fields.floats {
			self_floats.iter().for_each(|(k, v)| {
				floats.insert(k.clone(), *v).unwrap();
			});
		}

		let mut structures = HashMap::<FieldID, HeaplessBuffer, FnvBuildHasher>::default();
		if let Some(ref self_structures) = self.fields.structures {
			self_structures.iter().for_each(|(k, v)| {
				let vec = rmp_serde::to_vec(v).unwrap();
				structures.insert(k.clone(), HeaplessBuffer::from_slice(&vec.as_slice()).unwrap());
			});
		}

		GameObject {
			id,
			template: self.template,
			access_groups: self.access_groups,
			fields: GameObjectFields { longs, floats, structures },
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::template::{GameObjectFieldsTemplate, GameObjectTemplate, RoomTemplate, UserTemplate};

	impl RoomTemplate {
		pub fn create_user(&mut self, public_key: UserPublicKey, access_group: AccessGroups) -> UserPublicKey {
			self.users.push(UserTemplate {
				public_key,
				private_key: [5; 32],
				access_groups: access_group,
				objects: Option::None,
			});
			public_key
		}
	}

	#[test]
	fn should_convert_game_object() {
		let mut config_object = GameObjectTemplate {
			id: 100,
			template: 200,
			access_groups: Default::default(),
			fields: Default::default(),
		};
		config_object.fields.longs = Option::Some(Default::default());
		config_object.fields.floats = Option::Some(Default::default());
		config_object.fields.structures = Option::Some(Default::default());

		config_object.fields.longs.as_mut().unwrap().insert(0, 100);
		config_object.fields.floats.as_mut().unwrap().insert(1, 105.105);
		config_object
			.fields
			.structures
			.as_mut()
			.unwrap()
			.insert(1, rmpv::Value::Integer(rmpv::Integer::from(100100)));

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.owner, ObjectOwner::Root));
		assert_eq!(config_object.template, object.template);
		assert_eq!(config_object.access_groups, object.access_groups);
		assert_eq!(config_object.fields.longs.as_ref().unwrap()[&0], object.fields.longs[&0]);
		assert_eq!(config_object.fields.floats.as_ref().unwrap()[&1], object.fields.floats[&1]);

		assert_eq!(
			config_object.fields.structures.as_ref().unwrap()[&1],
			rmp_serde::from_slice(&object.fields.structures[&1].to_vec().as_slice()).unwrap()
		);
	}

	///
	/// генерация конфига для примера
	///
	#[allow(dead_code)]
	fn example() {
		let mut fields = GameObjectFieldsTemplate {
			longs: Option::Some(Default::default()),
			floats: Option::Some(Default::default()),
			structures: Option::Some(Default::default()),
		};

		fields.longs.as_mut().unwrap().insert(5, 100);
		fields.longs.as_mut().unwrap().insert(15, 200);

		fields.floats.as_mut().unwrap().insert(3, 5.5);
		fields.floats.as_mut().unwrap().insert(7, 9.9);

		fields.structures.as_mut().unwrap().insert(
			10,
			rmpv::Value::Map(vec![(
				rmpv::Value::String(rmpv::Utf8String::from("name")),
				rmpv::Value::String(rmpv::Utf8String::from("alex")),
			)]),
		);

		let config = RoomTemplate {
			id: 0,
			users: vec![UserTemplate {
				public_key: 54897,
				private_key: [5; 32],
				access_groups: AccessGroups(0b1111),
				objects: Option::Some(vec![GameObjectTemplate {
					id: 100,
					template: 0b100,
					access_groups: AccessGroups(0b1111),
					fields,
				}]),
			}],
			objects: Option::Some(vec![GameObjectTemplate {
				id: 5,
				template: 5,
				access_groups: Default::default(),
				fields: GameObjectFieldsTemplate {
					longs: Default::default(),
					floats: Default::default(),
					structures: Default::default(),
				},
			}]),
		};
		println!("{:}", serde_yaml::to_string(&config).unwrap());
	}
}
