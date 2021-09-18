use std::collections::HashMap;

use crate::proto::matches::relay::types as relay;
use crate::service::configurations::structures::{Field, FieldName, FieldType, GroupName, RoomName, RoomObject, Template, TemplateName};
use crate::service::resolver::error::Error;

///
/// Создаем объект для relay из конфигурации
///
pub fn create_relay_object(
	room_name: &RoomName,
	room_object: &RoomObject,
	templates: &HashMap<TemplateName, Template>,
	name_to_groups: &HashMap<GroupName, u64>,
	name_to_field: &HashMap<FieldName, Field>,
	next_object_id: u32,
) -> Result<relay::GameObjectTemplate, Error> {
	let template = templates
		.get(&room_object.template)
		.ok_or_else(|| Error::TemplateNotFound(room_name.clone(), room_object.template.clone()))?;

	let groups = name_to_groups
		.get(&room_object.group)
		.ok_or_else(|| Error::ObjectGroupNotFound(room_name.clone(), room_object.group.clone()))?;

	let mut relay_fields = relay::GameObjectFieldsTemplate {
		longs: Default::default(),
		floats: Default::default(),
		structures: Default::default(),
	};
	for value in &room_object.values {
		let field = name_to_field
			.get(&value.field)
			.ok_or_else(|| Error::FieldNotExistForObject(room_name.clone(), value.field.clone()))?;
		match field.r#type {
			FieldType::Long => {
				let value = value
					.value
					.as_i64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				relay_fields.longs.insert(field.id as u32, value);
			}
			FieldType::Double => {
				let value = value
					.value
					.as_f64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				relay_fields.floats.insert(field.id as u32, value);
			}
			FieldType::Struct => {
				let value = rmp_serde::to_vec(&value.value)
					.map_err(|_| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				relay_fields.structures.insert(field.id as u32, value);
			}
			FieldType::Event => {
				Result::Err(Error::EventValueNotSupported(room_name.to_string(), value.field.clone()))?;
			}
		}
	}

	Result::Ok(relay::GameObjectTemplate {
		id: if room_object.id > 0 { room_object.id } else { next_object_id },
		template: template.id,
		groups: *groups,
		fields: Option::Some(relay_fields),
	})
}

#[cfg(test)]
pub mod test {
	use std::collections::HashMap;

	use rmpv::Utf8String;

	use crate::proto::matches::relay::types as relay;
	use crate::service::configurations::structures::{
		Field, FieldType, FieldValue, GroupName, RoomObject, Template, TemplateName, TemplatePermissions,
	};
	use crate::service::resolver::error::Error;
	use crate::service::resolver::object::create_relay_object;

	#[test]
	pub fn should_create_relay_object() {
		let result = create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: 100,
				template: "template".to_string(),
				group: "red".to_string(),
				values: vec![
					FieldValue {
						field: "score".to_string(),
						value: rmpv::Value::Integer(rmpv::Integer::from(100)),
					},
					FieldValue {
						field: "healing".to_string(),
						value: rmpv::Value::F64(3.1),
					},
					FieldValue {
						field: "profile".to_string(),
						value: rmpv::Value::Map(vec![(
							rmpv::Value::String(Utf8String::from("f")),
							rmpv::Value::String(Utf8String::from("a")),
						)]),
					},
				],
			},
			&setup_templates(),
			&setup_groups(),
			&vec![
				(
					"score".to_string(),
					Field {
						id: 55,
						r#type: FieldType::Long,
					},
				),
				(
					"healing".to_string(),
					Field {
						id: 57,
						r#type: FieldType::Double,
					},
				),
				(
					"profile".to_string(),
					Field {
						id: 59,
						r#type: FieldType::Struct,
					},
				),
			]
			.into_iter()
			.collect(),
			0,
		);
		let object = result.unwrap();

		assert_eq!(object.template, 200);
		assert_eq!(object.id, 100);
		assert_eq!(object.groups, 4);
		assert_eq!(
			object.fields.unwrap(),
			relay::GameObjectFieldsTemplate {
				longs: vec![(55, 100)].into_iter().collect(),
				floats: vec![(57, 3.1)].into_iter().collect(),
				structures: vec![(59, vec![129, 161, 102, 161, 97])].into_iter().collect()
			}
		);
	}

	#[test]
	pub fn should_use_external_object_id() {
		let result = create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: 0,
				template: "template".to_string(),
				group: "red".to_string(),
				values: Default::default(),
			},
			&setup_templates(),
			&setup_groups(),
			&Default::default(),
			155,
		);
		let object = result.unwrap();
		assert_eq!(object.id, 155);
	}

	#[test]
	pub fn should_error_field_not_found() {
		let result = create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: 100,
				template: "template".to_string(),
				group: "red".to_string(),
				values: vec![FieldValue {
					field: "score".to_string(),
					value: rmpv::Value::Integer(rmpv::Integer::from(100)),
				}],
			},
			&setup_templates(),
			&setup_groups(),
			&Default::default(),
			0,
		);
		assert!(matches!(
				result,
				Result::Err(Error::FieldNotExistForObject(room_name, field_name))
				if room_name=="room" && field_name=="score"
		));
	}

	#[test]
	pub fn should_error_template_not_found() {
		assert!(matches!(
			create_relay_object(
				&"room".to_string(),
				&RoomObject {
					id: 100,
					template: "template".to_string(),
					group: "red".to_string(),
					values: vec![],
				},
				&HashMap::default(),
				&HashMap::default(),
				&HashMap::default(),
				0,
			),
			Result::Err(Error::TemplateNotFound(room_name, template_name))
			if room_name=="room" && template_name=="template"
		));
	}

	#[test]
	pub fn should_error_group_not_found() {
		let result = create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: 100,
				template: "template".to_string(),
				group: "red".to_string(),
				values: vec![],
			},
			&setup_templates(),
			&HashMap::default(),
			&HashMap::default(),
			0,
		);
		assert!(matches!(
			result,
			Result::Err(Error::ObjectGroupNotFound(room_name, group_name))
			if room_name=="room" && group_name=="red"
		));
	}

	fn setup_groups() -> HashMap<GroupName, u64> {
		vec![("red".to_string(), 4)].into_iter().collect()
	}

	fn setup_templates() -> HashMap<TemplateName, Template> {
		vec![(
			"template".to_string(),
			Template {
				id: 200,
				permissions: TemplatePermissions::default(),
			},
		)]
		.into_iter()
		.collect()
	}
}
