use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldIdType;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::room::object::GameObject;
use crate::room::template::config::GameObjectTemplate;
use crate::room::RoomId;

impl GameObjectTemplate {
	pub fn to_root_game_object(&self) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, ObjectOwner::Root))
	}
	pub fn to_user_game_object(&self, user_public_key: UserPublicKey) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, ObjectOwner::User(user_public_key)))
	}
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		if id.id == 0 {
			panic!("0 is forbidden for game object id");
		}

		let mut longs: HashMap<FieldIdType, i64, FnvBuildHasher> = Default::default();
		self.fields.longs.iter().for_each(|(k, v)| {
			longs.insert(k.clone(), *v);
		});

		let mut floats: HashMap<FieldIdType, f64, FnvBuildHasher> = Default::default();
		self.fields.floats.iter().for_each(|(k, v)| {
			floats.insert(k.clone(), *v);
		});

		let mut structures: HashMap<FieldIdType, Vec<u8>, FnvBuildHasher> = Default::default();
		self.fields.structures.iter().for_each(|(k, v)| {
			let structure = rmp_serde::to_vec(v).unwrap();
			structures.insert(k.clone(), structure);
		});

		GameObject {
			id,
			template: self.template,
			access_groups: self.access_groups,
			created: true,
			longs,
			floats,
			structures,
			compare_and_set_owners: Default::default(),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::room::template::config::GameObjectTemplate;
	use cheetah_relay_common::room::owner::ObjectOwner;

	#[test]
	#[should_panic]
	fn should_panic_if_object_id_is_0() {
		let config_object = GameObjectTemplate {
			id: 0,
			template: 200,
			access_groups: Default::default(),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		config_object.to_root_game_object();
	}

	#[test]
	fn should_convert_game_object() {
		let mut config_object = GameObjectTemplate {
			id: 100,
			template: 200,
			access_groups: Default::default(),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		config_object.fields.longs = Default::default();
		config_object.fields.floats = Default::default();
		config_object.fields.structures = Default::default();

		config_object.fields.longs.insert(0, 100);
		config_object.fields.floats.insert(1, 105.105);
		config_object
			.fields
			.structures
			.insert(1, rmpv::Value::Integer(rmpv::Integer::from(100100)));

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.owner, ObjectOwner::Root));
		assert_eq!(config_object.template, object.template);
		assert_eq!(config_object.access_groups, object.access_groups);
		assert_eq!(config_object.fields.longs[&0], object.longs[&0]);
		assert_eq!(config_object.fields.floats[&1], object.floats[&1]);

		assert_eq!(
			config_object.fields.structures[&1],
			rmp_serde::from_slice(&object.structures[&1].to_vec().as_slice()).unwrap()
		);
	}
}
