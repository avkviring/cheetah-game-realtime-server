use crate::server::room::object::GameObject;
use crate::server::room::template::config::GameObjectTemplate;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_game_realtime_protocol::RoomMemberId;

pub mod config;

impl GameObjectTemplate {
	#[must_use]
	pub fn to_root_game_object(&self) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Room))
	}

	#[must_use]
	pub fn create_member_game_object(&self, member_id: RoomMemberId) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Member(member_id)))
	}

	#[must_use]
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		assert_ne!(id.id, 0, "0 is forbidden for game object id");

		let mut object = GameObject::new(id, self.template, self.groups, true);

		self.longs.iter().for_each(|(&k, v)| object.long_fields.set(k, v.clone()));
		self.doubles.iter().for_each(|(&k, v)| object.double_fields.set(k, v.clone()));
		self.structures.iter().for_each(|(&k, v)| object.structure_fields.set(k, v.clone()));

		object
	}
}

#[cfg(test)]
mod tests {
	use crate::server::room::template::config::GameObjectTemplate;
	use cheetah_common::room::owner::GameObjectOwner;

	#[test]
	#[should_panic]
	fn should_panic_if_object_id_is_0() {
		let config_object = GameObjectTemplate {
			id: 0,
			template: 200,
			groups: Default::default(),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		let _object = config_object.to_root_game_object();
	}

	#[test]
	#[allow(clippy::float_cmp)]
	fn should_convert_game_object() {
		let mut config_object = GameObjectTemplate {
			id: 100,
			template: 200,
			groups: Default::default(),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};

		config_object.longs.insert(0, 100);
		config_object.doubles.insert(1, 105.105);
		config_object.structures.insert(2, [1].as_ref().into());

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.get_owner(), GameObjectOwner::Room));
		assert_eq!(config_object.template, object.template_id);
		assert_eq!(config_object.groups, object.access_groups);

		let config_value = config_object.longs.get(&0).unwrap();
		let object_value = object.long_fields.get(0).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value = config_object.doubles.get(&1).unwrap();
		let object_value = object.double_fields.get(1).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value = config_object.structures.get(&2).unwrap();
		let object_value = object.structure_fields.get(2).unwrap();
		assert_eq!(*config_value, *object_value);
	}
}
