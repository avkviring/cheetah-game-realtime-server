use std::collections::HashMap;

use crate::proto::matches::relay::types as relay;
use crate::service::configurations::structures::{Field, FieldName, GroupName, Permission, PermissionField, Template, TemplateName};
use crate::service::resolver::error::Error;

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
		r#type: relay::FieldType::from(&field.r#type) as i32,
		rules,
	})
}

fn create_permission_rule(
	template_name: &TemplateName,
	name_to_groups: &HashMap<GroupName, u64>,
	group: &GroupName,
	permission: &Permission,
) -> Result<relay::GroupsPermissionRule, Error> {
	let relay_permission = match permission {
		Permission::Deny => relay::PermissionLevel::Deny as i32,
		Permission::ReadOnly => relay::PermissionLevel::Ro as i32,
		Permission::ReadWrite => relay::PermissionLevel::Rw as i32,
	};

	name_to_groups
		.get(group)
		.copied()
		.map(|groups| relay::GroupsPermissionRule {
			groups,
			permission: relay_permission,
		})
		.ok_or_else(|| Error::GroupNotFoundInTemplate(template_name.clone(), group.clone()))
}

#[cfg(test)]
#[test]
fn resolver() {
	todo!()
	// use super::{PrefabField, Rule};
	//
	// let groups = {
	// 	let mut groups = HashMap::new();
	// 	groups.insert(Path::new("/dir/groups").into(), {
	// 		let mut file = HashMap::default();
	// 		file.insert("test".into(), 12345);
	// 		file
	// 	});
	// 	Groups::build(groups).1
	// };
	//
	// let mut access = HashMap::default();
	// access.insert("test".into(), Rule::Deny);
	//
	// let prefab = Prefab {
	// 	template: 4444,
	// 	groups: "/dir/groups".into(),
	// 	access: access.clone(),
	// 	fields: vec![
	// 		PrefabField {
	// 			name: "a".to_string(),
	// 			id: 1,
	// 			access: access.clone(),
	// 			field: OptionValue::Long { value: Some(7) },
	// 		},
	// 		PrefabField {
	// 			name: "b".to_string(),
	// 			id: 2,
	// 			access: access.clone(),
	// 			field: OptionValue::Long { value: None },
	// 		},
	// 		PrefabField {
	// 			name: "default".to_string(),
	// 			id: 3,
	// 			access: access.clone(),
	// 			field: OptionValue::Long { value: Some(22222) },
	// 		},
	// 	],
	// };
	//
	// let resolver = PrefabResolver::new(prefab, &groups, Path::new("")).unwrap();
	//
	// assert_eq!(resolver.template_id(), 4444);
	//
	// {
	// 	let base = vec![
	// 		ObjectField {
	// 			name: "a".into(),
	// 			value: FieldValue::Long { value: 12345 },
	// 		},
	// 		ObjectField {
	// 			name: "b".into(),
	// 			value: FieldValue::Long { value: 77777 },
	// 		},
	// 	];
	//
	// 	let extend = vec![ExtendField {
	// 		id: 4321,
	// 		value: FieldValue::Long { value: 99999 },
	// 	}];
	//
	// 	let obj = resolver.resolve(base, extend, Path::new("")).unwrap();
	//
	// 	assert_eq!(obj.longs[&1], 12345); // перезаписано
	// 	assert_eq!(obj.longs[&2], 77777); // установлено значение
	// 	assert_eq!(obj.longs[&3], 22222); // взято из префаба
	// 	assert_eq!(obj.longs[&4321], 99999); // добавлено из объекта
	// }
}
