use std::collections::HashMap;

use cheetah_matches_realtime_common::commands::field::FieldId;
use fnv::FnvBuildHasher;

use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::commands::FieldValue;
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::MemberPrivateKey;

use cheetah_matches_realtime_common::commands::field::Field;

///
/// Шаблон для создания комнаты
///
#[derive(Debug, Default, Clone)]
pub struct RoomTemplate {
	pub name: String,
	pub objects: Vec<GameObjectTemplate>,
	pub permissions: Permissions,
}

#[derive(Debug, Default, Clone)]
pub struct MemberTemplate {
	///
	/// Пользователь для которого игнорируются все настройки безопасности
	/// Обычно под данным пользователем подключаются плагины
	///
	pub super_member: bool,
	pub private_key: MemberPrivateKey,
	pub groups: AccessGroups,
	pub objects: Vec<GameObjectTemplate>,
}

#[derive(Debug, Default, Clone)]
pub struct GameObjectTemplate {
	pub id: u32,
	pub template: GameObjectTemplateId,
	pub groups: AccessGroups,
	pub fields: HashMap<(FieldId, FieldType), FieldValue, FnvBuildHasher>,
}

#[derive(Debug, Default, Clone)]
pub struct Permissions {
	pub templates: Vec<GameObjectTemplatePermission>,
}

#[derive(Debug, Default, Clone)]
pub struct GameObjectTemplatePermission {
	pub template: GameObjectTemplateId,
	pub rules: Vec<GroupsPermissionRule>,
	pub fields: Vec<PermissionField>,
}

#[derive(Debug, Copy, Clone)]
pub struct GroupsPermissionRule {
	pub groups: AccessGroups,
	pub permission: Permission,
}

#[derive(Debug, Clone)]
pub struct PermissionField {
	pub field: Field,
	pub rules: Vec<GroupsPermissionRule>,
}

#[derive(Debug, Clone, Copy, PartialEq, Ord, PartialOrd, Eq)]
pub enum Permission {
	Deny,
	Ro,
	Rw,
}

#[derive(Debug)]
pub enum UserTemplateError {
	UserObjectHasWrongId(MemberPrivateKey, u32),
}

impl MemberTemplate {
	#[must_use]
	pub fn new_member(groups: AccessGroups, objects: Vec<GameObjectTemplate>) -> Self {
		MemberTemplate::new(false, groups, objects)
	}

	#[must_use]
	pub fn new_super_member() -> Self {
		MemberTemplate::new(true, AccessGroups::super_group(), Default::default())
	}

	#[must_use]
	pub fn new_super_member_with_key(key: MemberPrivateKey) -> Self {
		let mut member = Self::new_super_member();
		member.private_key = key;
		member
	}

	fn new(super_member: bool, groups: AccessGroups, objects: Vec<GameObjectTemplate>) -> Self {
		MemberTemplate {
			super_member,
			private_key: MemberPrivateKey::new_random(),
			groups,
			objects,
		}
	}

	pub fn validate(self) -> Result<MemberTemplate, UserTemplateError> {
		for object in &self.objects {
			if object.id >= GameObjectId::CLIENT_OBJECT_ID_OFFSET {
				return Err(UserTemplateError::UserObjectHasWrongId(self.private_key, object.id));
			}
		}
		Ok(self)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::field::FieldId;
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;

	use crate::room::template::config::{
		GameObjectTemplate, GameObjectTemplatePermission, GroupsPermissionRule, MemberTemplate, Permission, PermissionField, Permissions,
		UserTemplateError,
	};
	use cheetah_matches_realtime_common::commands::field::Field;

	impl MemberTemplate {
		#[must_use]
		pub fn stub(access_group: AccessGroups) -> Self {
			MemberTemplate::new_member(access_group, Default::default())
		}

		pub fn configure_object(&mut self, id: u32, template: GameObjectTemplateId, access_groups: AccessGroups) -> &mut GameObjectTemplate {
			let objects = &mut self.objects;
			objects.push(GameObjectTemplate {
				id,
				template,
				groups: access_groups,
				fields: Default::default(),
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
					let template_permission = GameObjectTemplatePermission {
						template,
						rules: vec![],
						fields: vec![],
					};
					self.templates.push(template_permission);
					self.templates.iter_mut().find(|t| t.template == template).unwrap()
				}
				Some(template) => template,
			};

			let permission_field = match template_permission.fields.iter_mut().find(|f| f.field.id == *field_id) {
				None => {
					let permission_field = PermissionField {
						field: Field { id: *field_id, field_type },
						rules: vec![],
					};
					template_permission.fields.push(permission_field);
					template_permission.fields.iter_mut().find(|f| f.field.id == *field_id).unwrap()
				}
				Some(permission_field) => permission_field,
			};

			permission_field.rules.push(GroupsPermissionRule {
				groups: *access_group,
				permission,
			});
		}
	}

	#[test]
	fn should_validate_fail_when_user_object_has_wrong_id() {
		let objects = vec![GameObjectTemplate {
			id: GameObjectId::CLIENT_OBJECT_ID_OFFSET + 1,
			template: 0b100,
			groups: AccessGroups(0b1111),
			fields: Default::default(),
		}];
		let template = MemberTemplate::new_member(AccessGroups(0b1111), objects);
		assert!(matches!(template.validate(), Err(UserTemplateError::UserObjectHasWrongId(_, _))));
	}
}
