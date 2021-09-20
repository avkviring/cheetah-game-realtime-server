use std::collections::HashMap;
use std::convert::TryFrom;

use crate::proto::matches::relay::types as relay;
use crate::service::configurations::structures::{FieldType, Room};
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
				let auto_object_id_start = objects.iter().map(|o| o.id).max().unwrap_or(0);

				let objects = objects
					.iter()
					.enumerate()
					.map(|(index, o)| create_relay_object(room_name, o, templates, groups, fields, auto_object_id_start + index as u32))
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
