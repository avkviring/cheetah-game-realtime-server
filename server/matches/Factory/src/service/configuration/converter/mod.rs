use std::collections::HashMap;
use std::convert::TryFrom;

use crate::proto::matches::relay::internal as relay;
use crate::service::configuration::converter::error::Error;
use crate::service::configuration::converter::object::create_relay_object;
use crate::service::configuration::converter::template::create_template_permission;
use crate::service::configuration::yaml::structures::Room;
use crate::service::configuration::yaml::YamlConfigurations;

///
/// Преобразование текстовой конфигурации в grpc формат для relay сервера
///
pub mod error;
mod from;
mod object;
mod template;

const DEFAULT_OBJECT_ID_START: u32 = 1;

impl TryFrom<&YamlConfigurations> for HashMap<String, relay::RoomTemplate> {
	type Error = Error;

	fn try_from(value: &YamlConfigurations) -> Result<Self, Self::Error> {
		let YamlConfigurations {
			groups,
			fields,
			templates,
			rooms,
		} = value;
		rooms
			.iter()
			.map(|(template_name, room)| {
				tracing::info!("resolve room {:?}", template_name);

				let Room { objects } = room;
				//  смещение для генерации id объектов
				let auto_object_id_start = objects
					.iter()
					.map(|o| o.id.unwrap_or(DEFAULT_OBJECT_ID_START))
					.max()
					.unwrap_or(DEFAULT_OBJECT_ID_START);

				let objects = objects
					.iter()
					.enumerate()
					.map(|(index, o)| {
						create_relay_object(
							template_name,
							o,
							templates,
							groups,
							fields,
							auto_object_id_start + index as u32,
						)
					})
					.collect::<Result<_, Error>>()?;

				let permissions = templates
					.values()
					.map(|template| create_template_permission(template_name, template, groups, fields))
					.collect::<Result<_, Error>>()?;

				let relay_room = relay::RoomTemplate {
					template_name: template_name.clone(),
					objects,
					permissions: Some(relay::Permissions { objects: permissions }),
				};

				Ok((template_name.clone(), relay_room))
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::convert::TryFrom;

	use crate::proto::matches::relay::internal as relay;
	use crate::service::configuration::yaml::structures::{Room, RoomObject, Template};
	use crate::service::configuration::yaml::YamlConfigurations;

	#[test]
	fn should_auto_increment_start_not_zero() {
		let config = YamlConfigurations {
			groups: vec![("group".to_string(), 7)].into_iter().collect(),
			fields: Default::default(),
			templates: vec![(
				"template".to_string(),
				Template {
					id: 0,
					permissions: Default::default(),
				},
			)]
			.into_iter()
			.collect(),
			rooms: vec![(
				"name".to_string(),
				Room {
					objects: vec![RoomObject {
						id: None,
						template: "template".to_string(),
						group: "group".to_string(),
						values: vec![],
					}],
				},
			)]
			.into_iter()
			.collect(),
		};

		let result = HashMap::<String, relay::RoomTemplate>::try_from(&config).unwrap();
		let room = result.get(&"name".to_string()).unwrap();
		assert_eq!(room.objects.first().unwrap().id, 1);
	}
}
