use std::collections::HashMap;

use crate::proto::matches::realtime::internal as relay;
use crate::proto::matches::realtime::shared::GameObjectField;
use crate::service::configuration::converter::error::Error;
use crate::service::configuration::yaml::structures::{Field, FieldName, FieldType, GroupName, RoomName, RoomObject, Template, TemplateName};

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

	let mut relay_fields = Vec::with_capacity(room_object.values.len());
	for value in &room_object.values {
		let field = name_to_field
			.get(&value.field)
			.ok_or_else(|| Error::FieldNotExistForObject(room_name.clone(), value.field.clone()))?;
		match field.typ {
			FieldType::Long => {
				let value = value
					.value
					.as_i64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				let f = GameObjectField {
					id: field.id as u32,
					value: Some(value.into()),
				};
				relay_fields.push(f);
			}
			FieldType::Double => {
				let value = value
					.value
					.as_f64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				let f = GameObjectField {
					id: field.id as u32,
					value: Some(value.into()),
				};
				relay_fields.push(f);
			}
			FieldType::Struct => {
				let value = rmp_serde::to_vec(&value.value)
					.map_err(|_| Error::WrongFormatForFieldValue(room_name.clone(), value.field.clone(), value.value.to_string()))?;
				let f = GameObjectField {
					id: field.id as u32,
					value: Some(value.into()),
				};
				relay_fields.push(f);
			}
			FieldType::Event => {
				return Err(Error::EventValueNotSupported(room_name.to_string(), value.field.clone()));
			}
		}
	}

	Result::Ok(relay::GameObjectTemplate {
		id: match room_object.id {
			None => next_object_id,
			Some(id) => id,
		},
		template: template.id,
		groups: *groups,
		fields: relay_fields,
	})
}

#[cfg(test)]
pub mod test {
	use std::collections::HashMap;

	use rmpv::Utf8String;

	use crate::proto::matches::realtime::internal as relay;
	use crate::proto::matches::realtime::shared::GameObjectField;
	use crate::service::configuration::converter::error::Error;
	use crate::service::configuration::converter::object::create_relay_object;
	use crate::service::configuration::yaml::structures::{
		Field, FieldType, FieldValue, GroupName, RoomObject, Template, TemplateName, TemplatePermissions,
	};

	#[test]
	pub fn should_create_relay_object() {
		let result = setup(
			vec![
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
			vec![
				(
					"score",
					Field {
						name: None,
						id: 55,
						typ: FieldType::Long,
					},
				),
				(
					"healing",
					Field {
						name: None,
						id: 55,
						typ: FieldType::Double,
					},
				),
				(
					"profile",
					Field {
						name: None,
						id: 59,
						typ: FieldType::Struct,
					},
				),
			],
		);
		let object = result.unwrap();

		assert_eq!(object.template, 200);
		assert_eq!(object.id, 100);
		assert_eq!(object.groups, 4);
		assert_eq!(object.fields.len(), 3);
		assert_eq!(
			object.fields,
			[
				(GameObjectField {
					id: 55,
					value: Some(100.into()),
				}),
				(GameObjectField {
					id: 55,
					value: Some(3.1.into())
				}),
				(GameObjectField {
					id: 59,
					value: Some(vec![129, 161, 102, 161, 97].into())
				})
			]
		);
	}

	#[test]
	pub fn should_error_wrong_format_for_long_when_create_relay_object() {
		let result = setup(
			vec![FieldValue {
				field: "score".to_string(),
				value: rmpv::Value::F64(3.1),
			}],
			vec![(
				"score",
				Field {
					name: None,
					id: 55,
					typ: FieldType::Long,
				},
			)],
		);
		assert!(matches!(
			result,
			Result::Err(Error::WrongFormatForFieldValue(room_name, field_name, value))
			if room_name==*"room" && field_name==*"score" && value==*"3.1"
		))
	}

	#[test]
	pub fn should_error_for_event_when_create_relay_object() {
		let result = setup(
			vec![FieldValue {
				field: "score".to_string(),
				value: rmpv::Value::Nil,
			}],
			vec![(
				"score",
				Field {
					name: None,
					id: 0,
					typ: FieldType::Event,
				},
			)],
		);
		assert!(matches!(
			result,
			Result::Err(Error::EventValueNotSupported(room_name, field_name))
			if room_name==*"room" && field_name==*"score"
		))
	}

	#[test]
	pub fn should_create_relay_object_when_float_set_as_int() {
		let result = setup(
			vec![FieldValue {
				field: "score".to_string(),
				value: rmpv::Value::Integer(rmpv::Integer::from(123)),
			}],
			vec![(
				"score",
				Field {
					name: None,
					id: 55,
					typ: FieldType::Double,
				},
			)],
		);
		let result = result.unwrap();
		assert_eq!(
			result.fields,
			[(GameObjectField {
				id: 55,
				value: Some(123.0.into()),
			})]
		)
	}

	#[test]
	pub fn should_use_external_object_id() {
		let result = create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: None,
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
		let result = setup(
			vec![FieldValue {
				field: "score".to_string(),
				value: rmpv::Value::Integer(rmpv::Integer::from(100)),
			}],
			vec![],
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
					id: Some(100),
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
				id: Some(100),
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

	fn setup(field_values: Vec<FieldValue>, fields: Vec<(&str, Field)>) -> Result<relay::GameObjectTemplate, Error> {
		create_relay_object(
			&"room".to_string(),
			&RoomObject {
				id: Some(100),
				template: "template".to_string(),
				group: "red".to_string(),
				values: field_values,
			},
			&setup_templates(),
			&setup_groups(),
			&fields.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
			0,
		)
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
