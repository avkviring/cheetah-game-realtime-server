use fnv::{FnvHashMap, FnvHashSet};
use std::cell::RefCell;

use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;

use crate::room::template::config::{GroupsPermissionRule, Permission, Permissions};
use cheetah_matches_realtime_common::commands::field::Field;

#[derive(Debug, Default)]
pub struct PermissionManager {
	template_rules: FnvHashMap<GameObjectTemplateId, FnvHashMap<AccessGroups, Permission>>,
	field_rules: FnvHashMap<PermissionFieldKey, FnvHashMap<AccessGroups, Permission>>,

	write_access_template: FnvHashSet<GameObjectTemplateId>,
	write_access_fields: FnvHashSet<PermissionFieldKey>,

	cache: RefCell<FnvHashMap<PermissionCachedFieldKey, Permission>>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct PermissionFieldKey {
	template: GameObjectTemplateId,
	field: Field,
}

#[derive(Hash, Eq, PartialEq, Debug)]
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

			if entry.iter().any(|(_, &p)| p == Permission::Rw) {
				self.write_access_template.insert(template.template);
			} else {
				self.write_access_template.remove(&template.template);
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

				if entry.iter().any(|(_, &p)| p == Permission::Rw) {
					self.write_access_fields.insert(key);
				} else {
					self.write_access_fields.remove(&key);
				}
			}
		}
	}

	///
	/// Доступен ли объект на запись другим пользователем кроме создателя
	///
	#[must_use]
	pub fn has_write_access(&self, template: GameObjectTemplateId, field: Field) -> bool {
		self.write_access_template.contains(&template) || self.write_access_fields.contains(&PermissionFieldKey { template, field })
	}

	#[must_use]
	pub fn get_permission(&self, template: GameObjectTemplateId, field: Field, groups: AccessGroups) -> Permission {
		let field_key = PermissionFieldKey { template, field };

		*self
			.cache
			.borrow_mut()
			.entry(PermissionCachedFieldKey { field_key, groups })
			.or_insert_with(|| {
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
			.filter_map(
				|(group, &permission)| {
					if group.contains_any(&member_group) {
						Some(permission)
					} else {
						None
					}
				},
			)
			.max()
			.unwrap_or(Permission::Rw)
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::room::access::AccessGroups;

	use crate::room::template::config::{GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField, Permissions};
	use crate::room::template::permission::PermissionManager;
	use cheetah_matches_realtime_common::commands::field::Field;

	#[test]
	fn should_default_permission() {
		let permissions_manager = PermissionManager::new(&Permissions::default());
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

		let permissions_manager = PermissionManager::new(&permissions);

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

		let permissions_manager = PermissionManager::new(&permissions);

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
					field_type: FieldType::Long,
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
		let _ = permissions_manager.get_permission(
			10,
			Field {
				id: 10,
				field_type: FieldType::Long,
			},
			AccessGroups(0b01),
		);
		let _ = permissions_manager.get_permission(
			10,
			Field {
				id: 15,
				field_type: FieldType::Long,
			},
			AccessGroups(0b01),
		);
		// удаляем исходные данные
		permissions_manager.field_rules.clear();
		permissions_manager.template_rules.clear();

		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 10,
					field_type: FieldType::Long,
				},
				AccessGroups(0b01),
			),
			Permission::Deny
		);
		assert_eq!(
			permissions_manager.get_permission(
				10,
				Field {
					id: 15,
					field_type: FieldType::Long,
				},
				AccessGroups(0b01),
			),
			Permission::Rw
		);
	}

	#[test]
	fn should_not_has_write_access_by_default() {
		let permissions = Permissions::default();
		let permissions_manager = PermissionManager::new(&permissions);
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
		let permissions_manager = PermissionManager::new(&permissions);
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
		let permissions_manager = PermissionManager::new(&permissions);
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
		let permissions_manager = PermissionManager::new(&permissions);
		assert!(permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long,
			},
		));
	}

	#[test]
	fn should_update_permissions() {
		let template = 10;
		let field = Field {
			id: 100,
			field_type: FieldType::Long,
		};
		let groups = AccessGroups(1);

		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template,
			rules: vec![GroupsPermissionRule {
				groups,
				permission: Permission::Rw,
			}],
			fields: vec![],
		});
		let mut permissions_manager = PermissionManager::new(&permissions);

		// should have write access
		assert!(permissions_manager.has_write_access(template, field));
		assert_eq!(Permission::Rw, permissions_manager.get_permission(template, field, groups));

		// should not have write access after update
		permissions.templates[0].rules[0].permission = Permission::Ro;
		permissions_manager.update_permissions(&permissions);
		assert!(!permissions_manager.has_write_access(template, field));
		assert_eq!(Permission::Ro, permissions_manager.get_permission(template, field, groups));
	}

	#[test]
	fn should_deny_template() {
		let mut permissions = Permissions::default();
		permissions.templates.push(GameObjectTemplatePermission {
			template: 10,
			rules: vec![GroupsPermissionRule {
				groups: Default::default(),
				permission: Permission::Deny,
			}],
			fields: vec![],
		});

		// should not have write access
		let permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long,
			},
		));
	}

	#[test]
	fn should_deny_field() {
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
					permission: Permission::Deny,
				}],
			}],
		});

		// should have write access
		let permissions_manager = PermissionManager::new(&permissions);
		assert!(!permissions_manager.has_write_access(
			10,
			Field {
				id: 100,
				field_type: FieldType::Long,
			},
		));
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
			permissions_manager.get_permission(
				10,
				Field {
					id: 100,
					field_type: FieldType::Long,
				},
				AccessGroups(0b11),
			)
		);
	}
}
