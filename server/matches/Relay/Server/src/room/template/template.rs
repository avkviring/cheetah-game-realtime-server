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

		self.fields
			.iter()
			.for_each(|(&(k, t), v)| object.set_field(k, t, v.to_owned()).unwrap());

		object
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::{room::owner::GameObjectOwner, commands::FieldType};

	use crate::room::{field::FieldValue, template::config::GameObjectTemplate};

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

		config_object.fields.insert((0, FieldType::Long), FieldValue::Long(100));
		config_object.fields.insert((1, FieldType::Double), FieldValue::Double(105.105));
		config_object.fields.insert((2, FieldType::Structure), FieldValue::Structure(vec![1]));

		let object = config_object.clone().to_root_game_object();
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.owner, GameObjectOwner::Room));
		assert_eq!(config_object.template, object.template_id);
		assert_eq!(config_object.groups, object.access_groups);
		assert_eq!(config_object.fields[&(0, FieldType::Long)], *object.field(0, FieldType::Long).unwrap());
		assert_eq!(config_object.fields[&(1, FieldType::Double)], *object.field(1, FieldType::Double).unwrap());
		assert_eq!(config_object.fields[&(2, FieldType::Structure)], *object.field(2, FieldType::Structure).unwrap());
	}
}
