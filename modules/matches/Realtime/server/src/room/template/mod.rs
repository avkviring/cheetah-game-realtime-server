use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::object::GameObject;
use crate::room::template::config::GameObjectTemplate;

pub mod config;
pub mod permission;

impl GameObjectTemplate {
	#[must_use]
	pub fn to_root_game_object(&self) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Room))
	}

	#[must_use]
	pub fn create_user_game_object(&self, user_id: RoomMemberId) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Member(user_id)))
	}

	#[must_use]
	pub fn to_game_object(&self, id: GameObjectId) -> GameObject {
		assert_ne!(id.id, 0, "0 is forbidden for game object id");

		let mut object = GameObject::new(id, self.template, self.groups, true);

		self.fields
			.iter()
			.for_each(|(&(k, _), v)| object.set_field_wrapped(k, v.clone()).unwrap());

		object
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::{
		commands::{FieldType, FieldValue},
		room::owner::GameObjectOwner,
	};

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
		let _object = config_object.to_root_game_object();
	}

	#[test]
	#[allow(clippy::float_cmp)]
	fn should_convert_game_object() {
		let mut config_object = GameObjectTemplate {
			id: 100,
			template: 200,
			groups: Default::default(),
			fields: Default::default(),
		};

		config_object.fields.insert((0, FieldType::Long), FieldValue::Long(100));
		config_object.fields.insert((1, FieldType::Double), FieldValue::Double(105.105));
		config_object.fields.insert((2, FieldType::Structure), FieldValue::Structure(vec![1]));

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.owner, GameObjectOwner::Room));
		assert_eq!(config_object.template, object.template_id);
		assert_eq!(config_object.groups, object.access_groups);

		let config_value: &i64 = config_object.fields[&(0, FieldType::Long)].as_ref();
		let object_value: &i64 = object.get_field(0).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value: &f64 = config_object.fields[&(1, FieldType::Double)].as_ref();
		let object_value: &f64 = object.get_field(1).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value: &Vec<u8> = config_object.fields[&(2, FieldType::Structure)].as_ref();
		let object_value: &Vec<u8> = object.get_field(2).unwrap();
		assert_eq!(*config_value, *object_value);
	}
}
