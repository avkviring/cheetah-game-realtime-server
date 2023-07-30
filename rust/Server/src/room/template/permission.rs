use std::cell::RefCell;

use fnv::FnvHashMap;

use crate::room::template::config::{GroupsPermissionRule, Permission, Permissions};
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::field::Field;
use cheetah_common::room::object::GameObjectTemplateId;

#[derive(Debug, Default, Clone)]
pub struct PermissionManager {
	template_rules: FnvHashMap<GameObjectTemplateId, FnvHashMap<AccessGroups, Permission>>,
	field_rules: FnvHashMap<PermissionFieldKey, FnvHashMap<AccessGroups, Permission>>,
	cache: RefCell<FnvHashMap<PermissionCachedFieldKey, Permission>>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct PermissionFieldKey {
	template: GameObjectTemplateId,
	field: Field,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct PermissionCachedFieldKey {
	field_key: PermissionFieldKey,
	groups: AccessGroups,
}

impl PermissionManager {
	#[must_use]
	pub fn new(permissions: &Permissions) -> Self {
		let mut pm = Self::default();
		pm.update_permissions(permissions);
		pm
	}

	pub fn update_permissions(&mut self, permissions: &Permissions) {
		self.cache.borrow_mut().clear();
		for template in &permissions.templates {
			let entry = self.template_rules.entry(template.template).or_default();
			for GroupsPermissionRule { groups, permission } in &template.rules {
				entry.insert(*groups, *permission);
			}

			for field in &template.fields {
				let key = PermissionFieldKey {
					template: template.template,
					field: field.field,
				};

				let entry = self.field_rules.entry(key).or_default();
				for GroupsPermissionRule { groups, permission } in &field.rules {
					entry.insert(*groups, *permission);
				}
			}
		}
	}

	#[must_use]
	pub fn get_permission(&self, template: GameObjectTemplateId, field: Field, groups: AccessGroups) -> Permission {
		if groups == AccessGroups::super_member_group() {
			return Permission::Rw;
		}

		let field_key = PermissionFieldKey { template, field };

		*self.cache.borrow_mut().entry(PermissionCachedFieldKey { field_key, groups }).or_insert_with(|| {
			if let Some(field_rules) = self.field_rules.get(&field_key) {
				Self::get_permission_by_group(groups, field_rules)
			} else if let Some(template_rules) = self.template_rules.get(&template) {
				Self::get_permission_by_group(groups, template_rules)
			} else {
				Permission::Rw
			}
		})
	}

	fn get_permission_by_group(member_group: AccessGroups, groups: &FnvHashMap<AccessGroups, Permission>) -> Permission {
		groups
			.iter()
			.filter_map(|(group, &permission)| group.contains_any(&member_group).then_some(permission))
			.max()
			.unwrap_or(Permission::Rw)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::field::{Field, FieldType};
	use cheetah_common::room::object::GameObjectTemplateId;

	use crate::room::template::config::{GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField, Permissions};
	use crate::room::template::permission::PermissionManager;

	#[test]
	fn should_default_permission() {
		let permissions_manager = PermissionManager::new(&Permissions::default());
		assert_eq!(permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0)), Permission::Rw);
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

		let permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0b01)),
			Permission::Rw
		);
		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0b1000)),
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
			field: Field { id: 15, field_type: FieldType::Long },
			rules: vec![GroupsPermissionRule {
				groups: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let permissions_manager = PermissionManager::new(&permissions);

		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0b01)),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 15, field_type: FieldType::Long }, AccessGroups(0b01)),
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
			field: Field { id: 15, field_type: FieldType::Long },
			rules: vec![GroupsPermissionRule {
				groups: AccessGroups(0b11),
				permission: Permission::Rw,
			}],
		});
		permissions.templates.push(template_permission);

		let mut permissions_manager = PermissionManager::new(&permissions);
		// прогреваем кеш
		let _ = permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0b01));
		let _ = permissions_manager.get_permission(10, Field { id: 15, field_type: FieldType::Long }, AccessGroups(0b01));
		// удаляем исходные данные
		permissions_manager.field_rules.clear();
		permissions_manager.template_rules.clear();

		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 10, field_type: FieldType::Long }, AccessGroups(0b01),),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(10, Field { id: 15, field_type: FieldType::Long }, AccessGroups(0b01),),
			Permission::Rw
		);
	}

	#[test]
	fn should_update_permissions() {
		let template = 10;
		let field = Field { id: 100, field_type: FieldType::Long };
		let groups = AccessGroups(1);

		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template,
			rules: vec![GroupsPermissionRule { groups, permission: Permission::Rw }],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);

		// should have write access
		assert_eq!(Permission::Rw, permissions_manager.get_permission(template, field, groups));

		// should not have write access after update
		permissions.templates[0].rules[0].permission = Permission::Ro;
		permissions_manager.update_permissions(&permissions);
		assert_eq!(Permission::Ro, permissions_manager.get_permission(template, field, groups));
	}

	#[test]
	fn should_allow_for_super_user_field() {
		let permissions = Permissions::default();
		let permissions_manager = PermissionManager::new(&permissions);
		let permission = permissions_manager.get_permission(
			GameObjectTemplateId::default(),
			Field {
				id: Default::default(),
				field_type: FieldType::Double,
			},
			AccessGroups::super_member_group(),
		);
		assert_eq!(permission, Permission::Rw);
	}

	#[test]
	fn should_use_max_permission() {
		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template: 10,
			rules: vec![
				GroupsPermissionRule {
					groups: AccessGroups(0b1),
					permission: Permission::Ro,
				},
				GroupsPermissionRule {
					groups: AccessGroups(0b10),
					permission: Permission::Rw,
				},
			],
			fields: vec![],
		});

		let permissions_manager = PermissionManager::new(&permissions);
		assert_eq!(
			Permission::Rw,
			permissions_manager.get_permission(10, Field { id: 100, field_type: FieldType::Long }, AccessGroups(0b11),)
		);
	}
}
