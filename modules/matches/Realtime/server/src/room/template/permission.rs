use std::collections::{HashMap, HashSet};

use fnv::FnvBuildHasher;

use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;

use crate::room::object::Field;
use crate::room::template::config::{GroupsPermissionRule, Permission, Permissions};

#[derive(Debug)]
pub struct PermissionManager {
	templates: HashMap<GameObjectTemplateId, Vec<GroupsPermissionRule>, FnvBuildHasher>,
	fields: HashMap<PermissionFieldKey, Vec<GroupsPermissionRule>, FnvBuildHasher>,
	cache: HashMap<PermissionCachedFieldKey, Permission, FnvBuildHasher>,
	write_access_template: HashSet<GameObjectTemplateId, FnvBuildHasher>,
	write_access_fields: HashSet<PermissionFieldKey, FnvBuildHasher>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct PermissionFieldKey {
	template: GameObjectTemplateId,
	field: Field,
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
			write_access_template: Default::default(),
			write_access_fields: Default::default(),
		};

		for template in &permission.templates {
			if template
				.rules
				.iter()
				.any(|t| t.permission != Permission::Ro)
			{
				result.write_access_template.insert(template.template);
			}
			result
				.templates
				.insert(template.template, template.rules.clone());

			for field in &template.fields {
				let key = PermissionFieldKey {
					template: template.template,
					field: field.field,
				};

				if field.rules.iter().any(|t| t.permission != Permission::Ro) {
					result.write_access_fields.insert(key.clone());
				}

				result.fields.insert(key, field.rules.clone());
			}
		}

		result
	}

	///
	/// Доступен ли объект на запись другим пользователем кроме создателя
	///
	pub fn has_write_access(&mut self, template: GameObjectTemplateId, field: Field) -> bool {
		self.write_access_template.contains(&template)
			|| self
				.write_access_fields
				.contains(&PermissionFieldKey { template, field })
	}

	pub fn get_permission(
		&mut self,
		template: GameObjectTemplateId,
		field: Field,
		user_group: AccessGroups,
	) -> Permission {
		let field_key = PermissionFieldKey { template, field };

		let cached_key = PermissionCachedFieldKey {
			field_key,
			group: user_group,
		};

		match self.cache.get(&cached_key) {
			None => {
				let permission = match self.fields.get(&cached_key.field_key) {
					None => match self.templates.get(&template) {
						None => &Permission::Rw,
						Some(permissions) => {
							PermissionManager::get_permission_by_group(user_group, permissions)
						}
					},
					Some(permissions) => {
						PermissionManager::get_permission_by_group(user_group, permissions)
					}
				};
				self.cache.insert(cached_key, *permission);
				*permission
			}
			Some(permission) => *permission,
		}
	}

	fn get_permission_by_group(
		user_group: AccessGroups,
		groups: &[GroupsPermissionRule],
	) -> &Permission {
		groups
			.iter()
			.find(|p| p.groups.contains_any(&user_group))
			.map_or(&Permission::Rw, |p| &p.permission)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::room::access::AccessGroups;

	use crate::room::object::Field;
	use crate::room::template::config::{
		GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField,
		Permissions,
	};
	use crate::room::template::permission::PermissionManager;

	#[test]
	fn should_default_permission() {
		let mut permissions_manager = PermissionManager::new(&Permissions::default());
		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long
				},
				AccessGroups(0)
			),
			Permission::Rw
		);
	}

	#[test]
	fn should_permission_for_template_by_group() {
		let mut permissions = Permissions::default();
		let mut template_permission = GameObjectTemplatePermission {
			template: 10,
			rules: Default::default(),
			fields: Default::default(),
		};
		template_permission.rules.push(GroupsPermissionRule {
			groups: AccessGroups(0b11),
			permission: Permission::Rw,
		});

		template_permission.rules.push(GroupsPermissionRule {
			groups: AccessGroups(0b1000),
			permission: Permission::Deny,
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long
				},
				AccessGroups(0b01)
			),
			Permission::Rw
		);
		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long
				},
				AccessGroups(0b1000)
			),
			Permission::Deny
		);
	}

	#[test]
	fn should_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = GameObjectTemplatePermission {
			template: 10,
			rules: Default::default(),
			fields: Default::default(),
		};
		template_permission.rules.push(GroupsPermissionRule {
			groups: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field: Field {
				id: 15,
				field_type: FieldType::Long,
			},
			rules: vec![GroupsPermissionRule {
				groups: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long
				},
				AccessGroups(0b01)
			),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 15,
					field_type: FieldType::Long
				},
				AccessGroups(0b01)
			),
			Permission::Rw
		);
	}

	#[test]
	fn should_cache_permission_for_fields() {
		let mut permissions = Permissions::default();
		let mut template_permission = GameObjectTemplatePermission {
			template: 10,
			rules: Default::default(),
			fields: Default::default(),
		};
		template_permission.rules.push(GroupsPermissionRule {
			groups: AccessGroups(0b11),
			permission: Permission::Deny,
		});

		template_permission.fields.push(PermissionField {
			field: Field {
				id: 15,
				field_type: FieldType::Long,
			},
			rules: vec![GroupsPermissionRule {
				groups: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);
		// прогреваем кеш
		permissions_manager.get_permission(
			10,
			Field {
				id: 10,
				field_type: FieldType::Long,
			},
			AccessGroups(0b01),
		);
		permissions_manager.get_permission(
			10,
			Field {
				id: 15,
				field_type: FieldType::Long,
			},
			AccessGroups(0b01),
		);
		// удаляем исходные данные
		permissions_manager.fields.clear();
		permissions_manager.templates.clear();

		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long
				},
				AccessGroups(0b01)
			),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 15,
					field_type: FieldType::Long
				},
				AccessGroups(0b01)
			),
			Permission::Rw
		);
	}

	#[test]
	fn should_not_has_write_access_by_default() {
		let permissions = Permissions::default();
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long
			}
		));
	}

	#[test]
	fn should_has_write_access_if_object_has_write_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template: 10,
			rules: vec![GroupsPermissionRule {
				groups: Default::default(),
				permission: Permission::Rw,
			}],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long
			}
		));
	}

	#[test]
	fn should_not_has_write_access_if_object_has_read_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template: 10,
			rules: vec![GroupsPermissionRule {
				groups: Default::default(),
				permission: Permission::Ro,
			}],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long
			}
		));
	}

	#[test]
	fn should_has_write_access_if_object_has_field_with_write_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template: 10,
			rules: vec![],
			fields: vec![PermissionField {
				field: Field {
					id: 100,
					field_type: FieldType::Long,
				},
				rules: vec![GroupsPermissionRule {
					groups: Default::default(),
					permission: Permission::Rw,
				}],
			}],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);
		assert!(permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long,
			}
		));
	}
}
