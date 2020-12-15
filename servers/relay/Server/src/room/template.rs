use std::collections::HashMap;
use std::io::Read;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
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
	pub auto_create_user: bool,
	pub users: Vec<UserTemplate>,
	pub objects: Option<Vec<GameObjectTemplate>>,

	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserTemplate {
	pub public_key: UserPublicKey,
	pub private_key: UserPrivateKey,
	pub access_groups: AccessGroups,
	pub objects: Option<Vec<GameObjectTemplate>>,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectTemplate {
	pub id: u32,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFieldsTemplate,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectFieldsTemplate {
	pub longs: Option<HashMap<FieldID, i64, FnvBuildHasher>>,
	pub floats: Option<HashMap<FieldID, f64, FnvBuildHasher>>,
	pub structures: Option<HashMap<FieldID, rmpv::Value, FnvBuildHasher>>,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

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

		let mut longs: HashMap<FieldID, i64, FnvBuildHasher> = Default::default();
		if let Some(ref self_longs) = self.fields.longs {
			self_longs.iter().for_each(|(k, v)| {
				longs.insert(k.clone(), *v);
			});
		}

		let mut floats: HashMap<FieldID, f64, FnvBuildHasher> = Default::default();
		if let Some(ref self_floats) = self.fields.floats {
			self_floats.iter().for_each(|(k, v)| {
				floats.insert(k.clone(), *v);
			});
		}

		let mut structures: HashMap<FieldID, Vec<u8>, FnvBuildHasher> = Default::default();
		if let Some(ref self_structures) = self.fields.structures {
			self_structures.iter().for_each(|(k, v)| {
				let structure = rmp_serde::to_vec(v).unwrap();
				structures.insert(k.clone(), structure);
			});
		}

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

#[derive(Debug)]
pub enum RoomTemplateError {
	UserObjectHasWrongId(UserTemplate, u32),
	YamlParserError(serde_yaml::Error),
	YamlContainsUnmappingFields(Vec<String>),
}

impl RoomTemplate {
	pub fn load_from_file(path: &str) -> Result<RoomTemplate, RoomTemplateError> {
		let mut file = std::fs::File::open(path).unwrap();
		let mut content = String::default();
		file.read_to_string(&mut content).unwrap();
		RoomTemplate::new_from_yaml(content.as_str())
	}

	fn new_from_yaml(content: &str) -> Result<RoomTemplate, RoomTemplateError> {
		let template = serde_yaml::from_str::<RoomTemplate>(content);
		match template {
			Ok(template) => template.validate(),
			Err(e) => Result::Err(RoomTemplateError::YamlParserError(e)),
		}
	}

	pub fn validate(self) -> Result<RoomTemplate, RoomTemplateError> {
		let mut unmapping = Vec::new();

		self.unmapping.iter().for_each(|(key, _value)| unmapping.push(key.clone()));

		for user in &self.users {
			user.unmapping.iter().for_each(|(key, _value)| unmapping.push(format!("user/{}", key)));
			for object in user.objects.as_ref().unwrap_or(&Default::default()) {
				object
					.unmapping
					.iter()
					.for_each(|(key, _value)| unmapping.push(format!("user/object/{}", key)));

				object
					.fields
					.unmapping
					.iter()
					.for_each(|(key, _value)| unmapping.push(format!("user/object/fields/{}", key)));
				if object.id >= GameObjectId::CLIENT_OBJECT_ID_OFFSET {
					return Result::Err(RoomTemplateError::UserObjectHasWrongId(user.clone(), object.id));
				}
			}
		}

		match &self.objects {
			None => {}
			Some(objects) => {
				for object in objects {
					object.unmapping.iter().for_each(|(key, _value)| unmapping.push(format!("object/{}", key)));
					object
						.fields
						.unmapping
						.iter()
						.for_each(|(key, _value)| unmapping.push(format!("object/fields/{}", key)));
				}
			}
		}

		if unmapping.is_empty() {
			Result::Ok(self)
		} else {
			Result::Err(RoomTemplateError::YamlContainsUnmappingFields(unmapping))
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::template::{GameObjectFieldsTemplate, GameObjectTemplate, RoomTemplate, RoomTemplateError, UserTemplate};

	impl RoomTemplate {
		pub fn create_user(&mut self, public_key: UserPublicKey, access_group: AccessGroups) -> UserPublicKey {
			self.users.push(UserTemplate {
				public_key,
				private_key: [5; 32],
				access_groups: access_group,
				objects: Option::None,
				unmapping: Default::default(),
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
			unmapping: Default::default(),
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
		assert_eq!(config_object.fields.longs.as_ref().unwrap()[&0], object.longs[&0]);
		assert_eq!(config_object.fields.floats.as_ref().unwrap()[&1], object.floats[&1]);

		assert_eq!(
			config_object.fields.structures.as_ref().unwrap()[&1],
			rmp_serde::from_slice(&object.structures[&1].to_vec().as_slice()).unwrap()
		);
	}

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

	///
	/// генерация конфига для примера
	///
	#[allow(dead_code)]
	fn example() {
		let mut fields = GameObjectFieldsTemplate {
			longs: Option::Some(Default::default()),
			floats: Option::Some(Default::default()),
			structures: Option::Some(Default::default()),
			unmapping: Default::default(),
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

		let _ = RoomTemplate {
			id: 0,
			auto_create_user: false,
			users: vec![UserTemplate {
				public_key: 54897,
				private_key: [5; 32],
				access_groups: AccessGroups(0b1111),
				objects: Option::Some(vec![GameObjectTemplate {
					id: 100,
					template: 0b100,
					access_groups: AccessGroups(0b1111),
					fields,
					unmapping: Default::default(),
				}]),
				unmapping: Default::default(),
			}],
			objects: Option::Some(vec![GameObjectTemplate {
				id: 5,
				template: 5,
				access_groups: Default::default(),
				fields: GameObjectFieldsTemplate {
					longs: Default::default(),
					floats: Default::default(),
					structures: Default::default(),
					unmapping: Default::default(),
				},
				unmapping: Default::default(),
			}]),
			unmapping: Default::default(),
		};
	}

	#[test]
	fn should_validate_fail_when_user_object_has_wrong_id() {
		let template = RoomTemplate {
			id: 0,
			auto_create_user: false,
			users: vec![UserTemplate {
				public_key: 54897,
				private_key: [5; 32],
				access_groups: AccessGroups(0b1111),
				objects: Option::Some(vec![GameObjectTemplate {
					id: GameObjectId::CLIENT_OBJECT_ID_OFFSET + 1,
					template: 0b100,
					access_groups: AccessGroups(0b1111),
					fields: Default::default(),
					unmapping: Default::default(),
				}]),
				unmapping: Default::default(),
			}],
			objects: None,
			unmapping: Default::default(),
		};
		assert!(matches!(template.validate(), Result::Err(RoomTemplateError::UserObjectHasWrongId(_, _))))
	}

	#[test]
	fn should_fail_if_unmapping_field() {
		let mut template = RoomTemplate::default();
		template.unmapping.insert("wrong_field".to_string(), serde_yaml::Value::default());
		let mut user_template = UserTemplate {
			public_key: 0,
			private_key: Default::default(),
			access_groups: Default::default(),
			objects: Option::Some(Default::default()),
			unmapping: Default::default(),
		};
		user_template.unmapping.insert("wrong_field".to_string(), serde_yaml::Value::default());

		let mut object_template = GameObjectTemplate {
			id: 0,
			template: 0,
			access_groups: Default::default(),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		object_template.unmapping.insert("wrong_field".to_string(), serde_yaml::Value::default());
		object_template
			.fields
			.unmapping
			.insert("wrong_field".to_string(), serde_yaml::Value::default());
		user_template.objects.as_mut().unwrap().push(object_template.clone());

		template.users.push(user_template);

		template.objects = Option::Some(Default::default());
		template.objects.as_mut().unwrap().push(object_template);

		assert!(matches!(
			template.validate(),
			Result::Err(RoomTemplateError::YamlContainsUnmappingFields(fields))
			if fields[0] == "wrong_field"
			&& fields[1] == "user/wrong_field"
			&& fields[2] == "user/object/wrong_field"
			&& fields[3] == "user/object/fields/wrong_field"
			&& fields[4] == "object/wrong_field"
			&& fields[5] == "object/fields/wrong_field"
		))
	}
}
