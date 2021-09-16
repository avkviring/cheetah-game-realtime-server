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
			.get(&value.name)
			.ok_or_else(|| Error::FieldNotExistForObject(room_name.clone(), value.name.clone()))?;
		match field.r#type {
			FieldType::Long => {
				let value = value
					.value
					.as_i64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.name.clone(), value.value.to_string()))?;
				relay_fields.longs.insert(field.id as u32, value);
			}
			FieldType::Double => {
				let value = value
					.value
					.as_f64()
					.ok_or_else(|| Error::WrongFormatForFieldValue(room_name.clone(), value.name.clone(), value.value.to_string()))?;
				relay_fields.floats.insert(field.id as u32, value);
			}
			FieldType::Struct => {
				let value = rmp_serde::to_vec(&value.value)
					.map_err(|_| Error::WrongFormatForFieldValue(room_name.clone(), value.name.clone(), value.value.to_string()))?;
				relay_fields.structures.insert(field.id as u32, value);
			}
			FieldType::Event => {
				Result::Err(Error::EventValueNotSupported(room_name.clone(), value.name.clone()))?;
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
