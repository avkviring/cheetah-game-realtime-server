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
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Member(user_id)))
	}
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		if id.id == 0 {
			panic!("0 is forbidden for game object id");
		}

		let mut object = GameObject::new(id, self.template, self.groups, true);

		self.fields.longs.iter().for_each(|(k, v)| object.set_long(*k, *v).unwrap());
		self.fields
			.floats
			.iter()
			.for_each(|(k, v)| object.set_double(*k, *v).unwrap());
		self.fields.structures.iter().for_each(|(k, v)| {
			object.set_structure(*k, v.as_slice()).unwrap();
		});

		object
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
		assert_eq!(config_object.template, object.template_id);
		assert_eq!(config_object.groups, object.access_groups);
		assert_eq!(config_object.fields.longs[&0], object.get_long(&0).cloned().unwrap());
		assert!((config_object.fields.floats[&1] - object.get_double(&1).cloned().unwrap()).abs() < 0.001);
		assert_eq!(config_object.fields.structures[&1], *object.get_structure(&1).unwrap());
	}
}
