use std::collections::HashMap;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::object::GameObject;
use crate::room::template::config::GameObjectTemplate;

impl GameObjectTemplate {
	pub fn to_root_game_object(&self) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Room))
	}
	pub fn create_user_game_object(&self, user_id: RoomMemberId) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::User(user_id)))
	}
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		if id.id == 0 {
			panic!("0 is forbidden for game object id");
		}

		let mut longs: HashMap<FieldId, i64, FnvBuildHasher> = Default::default();
		self.fields.longs.iter().for_each(|(k, v)| {
			longs.insert(*k, *v);
		});

		let mut floats: HashMap<FieldId, f64, FnvBuildHasher> = Default::default();
		self.fields.floats.iter().for_each(|(k, v)| {
			floats.insert(*k, *v);
		});

		let mut structures: HashMap<FieldId, Vec<u8>, FnvBuildHasher> = Default::default();
		self.fields.structures.iter().for_each(|(k, v)| {
			structures.insert(*k, v.clone());
		});

		GameObject {
			id,
			template: self.template,
			access_groups: self.groups,
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
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::template::config::GameObjectTemplate;

	#[test]
	#[should_panic]
	fn should_panic_if_object_id_is_0() {
		let config_object = GameObjectTemplate {
			id: 0,
			template: 200,
			groups: Default::default(),
			fields: Default::default(),
		};
		config_object.to_root_game_object();
	}

	#[test]
	fn should_convert_game_object() {
		let mut config_object = GameObjectTemplate {
			id: 100,
			template: 200,
			groups: Default::default(),
			fields: Default::default(),
		};
		config_object.fields.longs = Default::default();
		config_object.fields.floats = Default::default();
		config_object.fields.structures = Default::default();

		config_object.fields.longs.insert(0, 100);
		config_object.fields.floats.insert(1, 105.105);
		config_object.fields.structures.insert(1, vec![1]);

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.owner, GameObjectOwner::Room));
		assert_eq!(config_object.template, object.template);
		assert_eq!(config_object.groups, object.access_groups);
		assert_eq!(config_object.fields.longs[&0], object.longs[&0]);
		assert_eq!(config_object.fields.floats[&1], object.floats[&1]);
		assert_eq!(config_object.fields.structures[&1], object.structures[&1]);
	}
}
