use rand::Rng;

use cheetah_matches_relay_common::room::access::AccessGroups;

use crate::proto::types as proto;
use crate::room::template::config;

impl From<proto::UserTemplate> for config::UserTemplate {
	fn from(source: proto::UserTemplate) -> Self {
		config::UserTemplate {
			private_key: rand::thread_rng().gen::<[u8; 32]>(),
			access_groups: AccessGroups(source.access_group),
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
		}
	}
}

impl From<proto::GameObjectTemplate> for config::GameObjectTemplate {
	fn from(source: proto::GameObjectTemplate) -> Self {
		config::GameObjectTemplate {
			id: source.id,
			template: source.template as u16,
			access_groups: AccessGroups(source.access_group),
			fields: config::GameObjectFieldsTemplate::from(source.fields),
		}
	}
}

impl From<Option<proto::FieldsTemplate>> for config::GameObjectFieldsTemplate {
	fn from(source: Option<proto::FieldsTemplate>) -> Self {
		let mut result = config::GameObjectFieldsTemplate {
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
		};
		match source {
			Some(field) => {
				for (id, value) in field.longs {
					result.longs.insert(id as u16, value);
				}
				for (id, value) in field.floats {
					result.floats.insert(id as u16, value);
				}
				for (id, value) in field.structures {
					result.structures.insert(id as u16, value);
				}
			}
			None => {}
		}
		return result;
	}
}

impl From<proto::RoomTemplate> for config::RoomTemplate {
	fn from(source: proto::RoomTemplate) -> config::RoomTemplate {
		config::RoomTemplate {
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
			permissions: config::Permissions::from(source.template_permissions),
		}
	}
}

impl From<Vec<proto::TemplatePermission>> for config::Permissions {
	fn from(source: Vec<proto::TemplatePermission>) -> Self {
		config::Permissions {
			templates: source.into_iter().map(config::TemplatePermission::from).collect(),
		}
	}
}

impl From<proto::TemplatePermission> for config::TemplatePermission {
	fn from(source: proto::TemplatePermission) -> Self {
		config::TemplatePermission {
			template: source.template as u16,
			groups: source.groups.into_iter().map(config::PermissionGroup::from).collect(),
			fields: source.fields.into_iter().map(config::PermissionField::from).collect(),
		}
	}
}

impl From<proto::PermissionGroup> for config::PermissionGroup {
	fn from(source: proto::PermissionGroup) -> Self {
		let deny = proto::Permission::Deny as i32;
		let ro = proto::Permission::Ro as i32;
		let rw = proto::Permission::Rw as i32;

		let permission = match source.permission {
			x if x == deny => config::Permission::Deny,
			x if x == ro => config::Permission::Ro,
			x if x == rw => config::Permission::Rw,
			_ => {
				panic!("Enum permission unrecognized {}", source.permission)
			}
		};
		config::PermissionGroup {
			group: AccessGroups(source.access_group),
			permission,
		}
	}
}

impl From<proto::PermissionField> for config::PermissionField {
	fn from(source: proto::PermissionField) -> Self {
		let event = proto::FieldType::Event as i32;
		let float = proto::FieldType::Float as i32;
		let long = proto::FieldType::Long as i32;
		let structure = proto::FieldType::Structure as i32;

		let field_type = match source.field_type {
			x if x == event => crate::room::types::FieldType::Event,
			x if x == float => crate::room::types::FieldType::Float,
			x if x == long => crate::room::types::FieldType::Long,
			x if x == structure => crate::room::types::FieldType::Structure,
			_ => {
				panic!("Enum field_type unrecognized {}", source.field_type)
			}
		};
		config::PermissionField {
			field_id: source.field_id as u16,
			field_type,
			groups: source.groups.into_iter().map(config::PermissionGroup::from).collect(),
		}
	}
}
