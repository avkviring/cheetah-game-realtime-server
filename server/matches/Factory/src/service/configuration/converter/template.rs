use std::collections::HashMap;

use crate::proto::matches::relay::internal as relay;
use crate::proto::matches::relay::shared;
use crate::service::configuration::converter::error::Error;
use crate::service::configuration::yaml::structures::{
	Field, FieldName, FieldType, GroupName, PermissionField, PermissionLevel, Template, TemplateName,
};

///
/// Создание описание прав доступа relay::GameObjectTemplatePermission из конфигурации
pub fn create_template_permission(
	template_name: &TemplateName,
	template: &Template,
	name_to_groups: &HashMap<GroupName, u64>,
	name_to_field: &HashMap<FieldName, Field>,
) -> Result<relay::GameObjectTemplatePermission, Error> {
	let rules = template
		.permissions
		.groups
		.iter()
		.map(|(group, permission)| create_permission_rule(template_name, name_to_groups, group, permission))
		.collect::<Result<_, Error>>()?;

	let fields = template
		.permissions
		.fields
		.iter()
		.map(|permission_field| create_permissions_field(template_name, name_to_groups, name_to_field, permission_field))
		.collect::<Result<_, Error>>()?;

	Result::Ok(relay::GameObjectTemplatePermission {
		template: template.id,
		rules,
		fields,
	})
}

fn create_permissions_field(
	template_name: &TemplateName,
	name_to_groups: &HashMap<GroupName, u64>,
	name_to_field: &HashMap<FieldName, Field>,
	permission_field: &PermissionField,
) -> Result<relay::PermissionField, Error> {
	let rules = permission_field
		.groups
		.iter()
		.map(|(group, permission)| create_permission_rule(template_name, name_to_groups, group, permission))
		.collect::<Result<_, Error>>()?;

	let field = name_to_field
		.get(&permission_field.field)
		.ok_or_else(|| Error::FieldNotExistsForTemplate(template_name.clone(), permission_field.field.clone()))?;

	Result::Ok(relay::PermissionField {
		id: field.id as u32,
		r#type: shared::FieldType::from(&field.r#type) as i32,
		rules,
	})
}

fn create_permission_rule(
	template_name: &TemplateName,
	name_to_groups: &HashMap<GroupName, u64>,
	group: &GroupName,
	permission: &PermissionLevel,
) -> Result<relay::GroupsPermissionRule, Error> {
	let relay_permission = relay::PermissionLevel::from(permission);

	name_to_groups
		.get(group)
		.copied()
		.map(|groups| relay::GroupsPermissionRule {
			groups,
			permission: relay_permission as i32,
		})
		.ok_or_else(|| Error::GroupNotFoundInTemplate(template_name.clone(), group.clone()))
}

impl From<&PermissionLevel> for relay::PermissionLevel {
	fn from(permission: &PermissionLevel) -> Self {
		match permission {
			PermissionLevel::Deny => relay::PermissionLevel::Deny,
			PermissionLevel::ReadOnly => relay::PermissionLevel::Ro,
			PermissionLevel::ReadWrite => relay::PermissionLevel::Rw,
		}
	}
}

impl From<&FieldType> for shared::FieldType {
	fn from(field_type: &FieldType) -> Self {
		match field_type {
			FieldType::Long => shared::FieldType::Long,
			FieldType::Double => shared::FieldType::Double,
			FieldType::Struct => shared::FieldType::Structure,
			FieldType::Event => shared::FieldType::Event,
		}
	}
}

#[cfg(test)]
pub mod test {
	use crate::proto::matches::relay::internal as relay;
	use crate::proto::matches::relay::shared;
	use crate::service::configuration::converter::error::Error;
	use crate::service::configuration::converter::template::{
		create_permission_rule, create_permissions_field, create_template_permission,
	};
	use crate::service::configuration::yaml::structures::{
		Field, FieldType, PermissionField, PermissionLevel, Template, TemplatePermissions,
	};

	#[test]
	fn should_create_template_permission() {
		let result = create_template_permission(
			&"template".to_string(),
			&Template {
				id: 155,
				permissions: TemplatePermissions {
					groups: vec![("groupA".to_string(), PermissionLevel::ReadOnly)].into_iter().collect(),
					fields: vec![PermissionField {
						field: "score".to_string(),
						groups: vec![("groupA".to_string(), PermissionLevel::ReadWrite)].into_iter().collect(),
					}],
				},
			},
			&vec![("groupA".to_string(), 5)].into_iter().collect(),
			&vec![(
				"score".to_string(),
				Field {
					name: None,
					id: 77,
					r#type: FieldType::Long,
				},
			)]
			.into_iter()
			.collect(),
		)
		.unwrap();

		assert_eq!(result.template, 155);
		assert_eq!(
			result.rules,
			vec![relay::GroupsPermissionRule {
				groups: 5,
				permission: relay::PermissionLevel::Ro as i32
			}]
		);
		assert_eq!(
			result.fields,
			vec![relay::PermissionField {
				id: 77,
				r#type: shared::FieldType::Long as i32,
				rules: vec![relay::GroupsPermissionRule {
					groups: 5,
					permission: relay::PermissionLevel::Rw as i32
				}]
			}]
		)
	}

	#[test]
	fn should_error_group_not_found_when_create_template_permission() {
		let result = create_template_permission(
			&"template".to_string(),
			&Template {
				id: 155,
				permissions: TemplatePermissions {
					groups: vec![("groupA".to_string(), PermissionLevel::ReadOnly)].into_iter().collect(),
					fields: Default::default(),
				},
			},
			&Default::default(),
			&Default::default(),
		);
		assert!(matches!(result, 
			Result::Err(Error::GroupNotFoundInTemplate(template_name, group_name)) 
			if template_name==*"template" && group_name==*"groupA"));
	}

	#[test]
	fn should_error_field_not_found_when_create_template_permission() {
		let result = create_template_permission(
			&"template".to_string(),
			&Template {
				id: 155,
				permissions: TemplatePermissions {
					groups: Default::default(),
					fields: vec![PermissionField {
						field: "score".to_string(),
						groups: Default::default(),
					}],
				},
			},
			&Default::default(),
			&Default::default(),
		);
		assert!(matches!(result, 
			Result::Err(Error::FieldNotExistsForTemplate(template_name,group_name ))
			if template_name==*"template" && group_name==*"score"));
	}

	#[test]
	fn should_convert_field_type() {
		assert_eq!(shared::FieldType::from(&FieldType::Long), shared::FieldType::Long);
		assert_eq!(shared::FieldType::from(&FieldType::Struct), shared::FieldType::Structure);
		assert_eq!(shared::FieldType::from(&FieldType::Double), shared::FieldType::Double);
		assert_eq!(shared::FieldType::from(&FieldType::Event), shared::FieldType::Event);
	}

	#[test]
	fn should_convert_permission_level() {
		assert_eq!(
			relay::PermissionLevel::from(&PermissionLevel::ReadOnly),
			relay::PermissionLevel::Ro
		);
		assert_eq!(
			relay::PermissionLevel::from(&PermissionLevel::ReadWrite),
			relay::PermissionLevel::Rw
		);
		assert_eq!(
			relay::PermissionLevel::from(&PermissionLevel::Deny),
			relay::PermissionLevel::Deny
		);
	}

	#[test]
	pub fn should_create_permissions_field() {
		let result = create_permissions_field(
			&"template".to_string(),
			&vec![("groupA".to_string(), 64)].into_iter().collect(),
			&vec![(
				"score".to_string(),
				Field {
					name: None,
					id: 128,
					r#type: FieldType::Long,
				},
			)]
			.into_iter()
			.collect(),
			&PermissionField {
				field: "score".to_string(),
				groups: vec![("groupA".to_string(), PermissionLevel::ReadOnly)].into_iter().collect(),
			},
		);
		let result = result.unwrap();
		assert_eq!(result.id, 128);
		assert_eq!(result.r#type, shared::FieldType::Long as i32);
		assert_eq!(
			result.rules,
			vec![relay::GroupsPermissionRule {
				groups: 64,
				permission: relay::PermissionLevel::Ro as i32
			}]
		)
	}

	#[test]
	pub fn should_error_field_not_found_when_create_permissions_field() {
		let result = create_permissions_field(
			&"template".to_string(),
			&vec![("groupA".to_string(), 64)].into_iter().collect(),
			&Default::default(),
			&PermissionField {
				field: "score".to_string(),
				groups: vec![("groupA".to_string(), PermissionLevel::ReadOnly)].into_iter().collect(),
			},
		);
		assert!(matches!(result,
			Result::Err(Error::FieldNotExistsForTemplate(template_name,field_name))
			if template_name=="template" && field_name=="score"));
	}

	#[test]
	pub fn should_error_group_not_found_when_create_permissions_field() {
		let result = create_permissions_field(
			&"template".to_string(),
			&Default::default(),
			&Default::default(),
			&PermissionField {
				field: "score".to_string(),
				groups: vec![("groupA".to_string(), PermissionLevel::ReadOnly)].into_iter().collect(),
			},
		);
		assert!(matches!(result,
			Result::Err(Error::GroupNotFoundInTemplate(template_name,group_name))
			if template_name=="template" && group_name=="groupA"));
	}

	#[test]
	pub fn should_create_permission_rule() {
		let result = create_permission_rule(
			&"template".to_string(),
			&vec![("group".to_string(), 64)].into_iter().collect(),
			&"group".to_string(),
			&PermissionLevel::ReadWrite,
		);

		let result = result.unwrap();
		assert_eq!(result.groups, 64);
		assert_eq!(result.permission, relay::PermissionLevel::Rw as i32)
	}
	#[test]
	pub fn should_group_not_found_when_create_permission_rule() {
		let result = create_permission_rule(
			&"template".to_string(),
			&Default::default(),
			&"group".to_string(),
			&PermissionLevel::ReadOnly,
		);
		assert!(matches!(result,
			Result::Err(Error::GroupNotFoundInTemplate(template_name,group_name))
				if template_name=="template" && group_name=="group"));
	}
}
