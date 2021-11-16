use std::collections::HashMap;
use std::convert::TryFrom;

use crate::proto::matches::relay::internal as relay;
use crate::service::configurations::structures::Room;
use crate::service::configurations::Configurations;
use crate::service::resolver::error::Error;
use crate::service::resolver::object::create_relay_object;
use crate::service::resolver::template::create_template_permission;

///
/// Преобразование текстовой конфигурации в grpc формат для relay сервера
///
pub mod error;
mod object;
mod template;

const DEFAULT_OBJECT_ID_START: u32 = 1;

impl TryFrom<&Configurations> for HashMap<String, relay::RoomTemplate> {
	type Error = Error;

	fn try_from(value: &Configurations) -> Result<Self, Self::Error> {
		let Configurations {
			groups,
			fields,
			templates,
			rooms,
		} = value;
		rooms
			.iter()
			.map(|(room_name, room)| {
				log::info!("resolve room {:?}", room_name);

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
						create_relay_object(room_name, o, templates, groups, fields, auto_object_id_start + index as u32)
					})
					.collect::<Result<_, Error>>()?;

				let permissions = templates
					.values()
					.map(|template| create_template_permission(room_name, template, groups, fields))
					.collect::<Result<_, Error>>()?;

				let relay_room = relay::RoomTemplate {
					objects,
					permissions: Some(relay::Permissions { objects: permissions }),
				};

				Ok((room_name.clone(), relay_room))
			})
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::convert::TryFrom;

	use crate::proto::matches::relay::internal as relay;
	use crate::service::configurations::structures::{Room, RoomObject, Template};
	use crate::service::configurations::Configurations;

	#[test]
	fn should_auto_increment_start_not_zero() {
		let config = Configurations {
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
