use std::collections::HashMap;
use std::io::Read;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};

use crate::room::types::FieldType;
use crate::room::RoomId;

///
/// Шаблон для создания комнаты
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RoomTemplate {
	pub id: RoomId,
	pub auto_create_user: bool,
	pub users: Vec<UserTemplate>,
	#[serde(default)]
	pub objects: Vec<GameObjectTemplate>,
	#[serde(default)]
	pub permissions: Permissions,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserTemplate {
	pub public_key: UserPublicKey,
	pub private_key: UserPrivateKey,
	pub access_groups: AccessGroups,
	#[serde(default)]
	pub objects: Vec<GameObjectTemplate>,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectTemplate {
	pub id: u32,
	pub template: GameObjectTemplateId,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFieldsTemplate,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct GameObjectFieldsTemplate {
	#[serde(default)]
	pub longs: HashMap<FieldId, i64, FnvBuildHasher>,
	#[serde(default)]
	pub floats: HashMap<FieldId, f64, FnvBuildHasher>,
	#[serde(default)]
	pub structures: HashMap<FieldId, rmpv::Value, FnvBuildHasher>,
	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Permissions {
	#[serde(default)]
	pub templates: Vec<TemplatePermission>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TemplatePermission {
	pub template: GameObjectTemplateId,
	#[serde(default)]
	pub groups: Vec<PermissionGroup>,
	#[serde(default)]
	pub fields: Vec<PermissionField>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct PermissionGroup {
	pub group: AccessGroups,
	pub permission: Permission,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionField {
	pub field_id: FieldId,
	pub field_type: FieldType,
	#[serde(default)]
	pub groups: Vec<PermissionGroup>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
#[repr(u8)]
pub enum Permission {
	#[serde(rename = "deny")]
	Deny,
	#[serde(rename = "ro")]
	Ro,
	#[serde(rename = "rw")]
	Rw,
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
			for object in &user.objects {
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

		for object in &self.objects {
			object
				.unmapping
				.iter()
				.for_each(|(key, _value)| unmapping.push(format!("object/{}", key)));
			object
				.fields
				.unmapping
				.iter()
				.for_each(|(key, _value)| unmapping.push(format!("object/fields/{}", key)));
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
	use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::object::GameObject;
	use crate::room::template::config::{
		GameObjectTemplate, Permission, PermissionField, PermissionGroup, Permissions, RoomTemplate, RoomTemplateError, TemplatePermission,
		UserTemplate,
	};
	use crate::room::types::FieldType;

	impl RoomTemplate {
		pub fn configure_user(&mut self, public_key: UserPublicKey, access_group: AccessGroups) -> UserPublicKey {
			self.users.push(UserTemplate {
				public_key,
				private_key: [5; 32],
				access_groups: access_group,
				objects: Default::default(),
				unmapping: Default::default(),
			});
			public_key
		}

		pub fn configure_user_object(
			&mut self,
			id: u32,
			user_id: &UserPublicKey,
			template: GameObjectTemplateId,
			access_groups: AccessGroups,
		) -> &mut GameObjectTemplate {
			match self.users.iter_mut().find(|t| t.public_key == *user_id) {
				None => {
					panic!("user({}) not found", user_id);
				}
				Some(user) => {
					let objects = &mut user.objects;
					objects.push(GameObjectTemplate {
						id,
						template,
						access_groups,
						fields: Default::default(),
						unmapping: Default::default(),
					});
					let len = objects.len();
					let option = objects.get_mut(len - 1);
					option.unwrap()
				}
			}
		}
	}

	impl Permissions {
		pub fn set_permission(
			&mut self,
			template: GameObjectTemplateId,
			field_id: &FieldId,
			field_type: FieldType,
			access_group: &AccessGroups,
			permission: Permission,
		) {
			let template_permission = match self.templates.iter_mut().find(|t| t.template == template) {
				None => {
					let template_permission = TemplatePermission {
						template,
						groups: vec![],
						fields: vec![],
					};
					self.templates.push(template_permission);
					self.templates.iter_mut().find(|t| t.template == template).unwrap()
				}
				Some(template) => template,
			};

			let permission_field = match template_permission.fields.iter_mut().find(|f| f.field_id == *field_id) {
				None => {
					let permission_field = PermissionField {
						field_id: field_id.clone(),
						field_type,
						groups: vec![],
					};
					template_permission.fields.push(permission_field);
					template_permission.fields.iter_mut().find(|f| f.field_id == *field_id).unwrap()
				}
				Some(permission_field) => permission_field,
			};

			permission_field.groups.push(PermissionGroup {
				group: access_group.clone(),
				permission,
			});
		}
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
				objects: vec![GameObjectTemplate {
					id: GameObjectId::CLIENT_OBJECT_ID_OFFSET + 1,
					template: 0b100,
					access_groups: AccessGroups(0b1111),
					fields: Default::default(),
					unmapping: Default::default(),
				}],
				unmapping: Default::default(),
			}],
			objects: Default::default(),
			permissions: Default::default(),
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
			objects: Default::default(),
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
		user_template.objects.push(object_template.clone());

		template.users.push(user_template);

		template.objects = Default::default();
		template.objects.push(object_template);

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
