use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::{RoomId, UserId, UserPrivateKey};

use crate::room::types::FieldType;

///
/// Шаблон для создания комнаты
///
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RoomTemplate {
	pub id: RoomId,
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
	pub id: UserId,
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
	#[serde(default)]
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
}

impl RoomTemplate {
	pub fn validate(self) -> Result<RoomTemplate, RoomTemplateError> {
		for user in &self.users {
			for object in &user.objects {
				if object.id >= GameObjectId::CLIENT_OBJECT_ID_OFFSET {
					return Result::Err(RoomTemplateError::UserObjectHasWrongId(user.clone(), object.id));
				}
			}
		}
		Result::Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::UserId;

	use crate::room::template::config::{
		GameObjectTemplate, Permission, PermissionField, PermissionGroup, Permissions, RoomTemplate, RoomTemplateError, TemplatePermission,
		UserTemplate,
	};
	use crate::room::types::FieldType;

	impl RoomTemplate {
		pub fn configure_user(&mut self, user_id: UserId, access_group: AccessGroups) -> &mut UserTemplate {
			self.users.push(UserTemplate {
				id: user_id,
				private_key: [5; 32],
				access_groups: access_group,
				objects: Default::default(),
				unmapping: Default::default(),
			});
			let len = self.users.len();
			self.users.get_mut(len - 1).unwrap()
		}
	}

	impl UserTemplate {
		pub fn configure_object(&mut self, id: u32, template: GameObjectTemplateId, access_groups: AccessGroups) -> &mut GameObjectTemplate {
			let objects = &mut self.objects;
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
			users: vec![UserTemplate {
				id: 54897,
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
}
