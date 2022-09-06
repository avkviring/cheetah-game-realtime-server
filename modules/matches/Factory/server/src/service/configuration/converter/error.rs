use crate::service::configuration::yaml::structures::{FieldName, GroupName, RoomName, TemplateName};

#[derive(Debug)]
pub enum Error {
	TemplateNotFound(RoomName, TemplateName),
	ObjectGroupNotFound(RoomName, GroupName),
	FieldNotExistsForTemplate(TemplateName, FieldName),
	FieldNotExistForObject(RoomName, FieldName),
	GroupNotFoundInTemplate(TemplateName, GroupName),
	WrongFormatForFieldValue(RoomName, FieldName, String),
	EventValueNotSupported(RoomName, FieldName),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::TemplateNotFound(room, prefab) => {
				write!(f, "{}: Template {} not found", room, prefab,)
			}
			Error::ObjectGroupNotFound(room, group) => {
				write!(f, "Group {} not found in room {}", group, room)
			}
			Error::FieldNotExistForObject(room_name, field_name) => {
				write!(f, "Field {} not found in room {}", field_name, room_name)
			}
			Error::FieldNotExistsForTemplate(template, field_name) => {
				write!(f, "Field {} not found in template {}", field_name, template)
			}
			Error::GroupNotFoundInTemplate(template, group) => {
				write!(f, "Group {} not found in template {}", group, template)
			}
			Error::WrongFormatForFieldValue(room_name, field_name, value) => {
				write!(f, "Wrong format value \"{}\" for field {} in room {}", value, field_name, room_name)
			}
			Error::EventValueNotSupported(room_name, field_name) => {
				write!(f, "Set value for event {} not supported in room {}", field_name, room_name)
			}
		}
	}
}
