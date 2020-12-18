use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::{FieldIdType, GameObjectTemplateType};
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::template::config::{GameObjectTemplate, Permission, PermissionGroup, Permissions, TemplatePermission};
use crate::room::types::FieldType;

#[derive(Debug)]
pub struct PermissionManager {
	templates: HashMap<GameObjectTemplateType, Vec<PermissionGroup>, FnvBuildHasher>,
	fields: HashMap<PermissionFieldKey, Vec<PermissionGroup>, FnvBuildHasher>,
	cache: HashMap<PermissionCachedFieldKey, Permission, FnvBuildHasher>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct PermissionFieldKey {
	template: GameObjectTemplateType,
	field_id: FieldIdType,
	field_type: FieldType,
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct PermissionCachedFieldKey {
	field_key: PermissionFieldKey,
	group: AccessGroups,
}

impl PermissionManager {
	pub fn new(permission: &Permissions) -> Self {
		let mut result = Self {
			templates: Default::default(),
			fields: Default::default(),
			cache: Default::default(),
		};

		for template in &permission.templates {
			result.templates.insert(template.template.clone(), template.groups.clone());
			for field in &template.fields {
				let key = PermissionFieldKey {
					template: template.template,
					field_id: field.field_id,
					field_type: field.field_type.clone(),
				};
				result.fields.insert(key, field.groups.clone());
			}
		}

		result
	}

	///
	/// Доступен ли объект на запись другим пользователем кроме создателя
	///
	pub fn has_write_access(&mut self, template: GameObjectTemplateType, field_id: FieldIdType, field_type: FieldType) -> bool {
		unimplemented!()
	}

	pub fn get_permission(
		&mut self,
		template: GameObjectTemplateType,
		field_id: FieldIdType,
		field_type: FieldType,
		user_group: AccessGroups,
	) -> Permission {
		let field_key = PermissionFieldKey {
			template,
			field_id,
			field_type,
		};

		let cached_key = PermissionCachedFieldKey {
			field_key,
			group: user_group,
		};

		match self.cache.get(&cached_key) {
			None => {
				let permission = match self.fields.get(&cached_key.field_key) {
					None => match self.templates.get(&template) {
						None => &Permission::Ro,
						Some(permissions) => PermissionManager::get_permission_by_group(user_group, permissions),
					},
					Some(permissions) => PermissionManager::get_permission_by_group(user_group, permissions),
				};
				self.cache.insert(cached_key, *permission);
				*permission
			}
			Some(permission) => *permission,
		}
	}

	fn get_permission_by_group(user_group: AccessGroups, groups: &Vec<PermissionGroup>) -> &Permission {
		groups
			.iter()
			.find(|p| p.group.contains_any(&user_group))
			.map_or(&Permission::Ro, |p| &p.permission)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::template::config::{Permission, PermissionField, PermissionGroup, Permissions, TemplatePermission};
	use crate::room::template::permission::PermissionManager;
	use crate::room::types::FieldType;

	#[test]
	fn should_default_permission() {
		let mut permissions_manager = PermissionManager::new(&Permissions::default());
		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0)),
			Permission::Ro
		);
	}

	#[test]
	fn should_permission_for_template_by_group() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Rw,
		});

		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b1000),
			permission: Permission::Deny,
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b1000)),
			Permission::Deny
		);
	}

	#[test]
	fn should_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field_id: 15,
			field_type: FieldType::Long,
			groups: vec![PermissionGroup {
				group: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
	}

	#[test]
	fn should_cache_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = TemplatePermission {
			template: 10,
			groups: Default::default(),
			fields: Default::default(),
		};
		template_permission.groups.push(PermissionGroup {
			group: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field_id: 15,
			field_type: FieldType::Long,
			groups: vec![PermissionGroup {
				group: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);
		// прогреваем кеш
		permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01));
		permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01));
		// удаляем исходные данные
		permissions_manager.fields.clear();
		permissions_manager.templates.clear();

		assert_eq!(
			permissions_manager.get_permission(10, 10, FieldType::Long, AccessGroups(0b01)),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, 15, FieldType::Long, AccessGroups(0b01)),
			Permission::Rw
		);
	}
}
