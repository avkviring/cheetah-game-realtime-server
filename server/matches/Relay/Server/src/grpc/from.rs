use rand::Rng;

use cheetah_matches_relay_common::room::access::AccessGroups;

use crate::proto::types as proto;
use crate::room::template::config;

impl From<proto::RoomTemplate> for config::RoomTemplate {
	fn from(source: proto::RoomTemplate) -> config::RoomTemplate {
		config::RoomTemplate {
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
			permissions: config::Permissions::from(source.permissions.unwrap_or(proto::Permissions::default())),
		}
	}
}

impl From<proto::UserTemplate> for config::UserTemplate {
	fn from(source: proto::UserTemplate) -> Self {
		config::UserTemplate {
			private_key: rand::thread_rng().gen::<[u8; 32]>(),
			groups: AccessGroups(source.groups),
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
		}
	}
}

impl From<proto::GameObjectTemplate> for config::GameObjectTemplate {
	fn from(source: proto::GameObjectTemplate) -> Self {
		config::GameObjectTemplate {
			id: source.id,
			template: source.template as u16,
			groups: AccessGroups(source.groups),
			fields: config::GameObjectFieldsTemplate::from(source.fields.unwrap_or(proto::GameObjectFieldsTemplate::default())),
		}
	}
}

impl From<proto::GameObjectFieldsTemplate> for config::GameObjectFieldsTemplate {
	fn from(source: proto::GameObjectFieldsTemplate) -> Self {
		config::GameObjectFieldsTemplate {
			longs: source.longs.into_iter().map(|(k, v)| (k as u16, v)).collect(),
			floats: source.floats.into_iter().map(|(k, v)| (k as u16, v)).collect(),
			structures: source.structures.into_iter().map(|(k, v)| (k as u16, v)).collect(),
		}
	}
}

impl From<proto::Permissions> for config::Permissions {
	fn from(source: proto::Permissions) -> Self {
		config::Permissions {
			templates: source.objects.into_iter().map(config::GameObjectTemplatePermission::from).collect(),
		}
	}
}

impl From<proto::GameObjectTemplatePermission> for config::GameObjectTemplatePermission {
	fn from(source: proto::GameObjectTemplatePermission) -> Self {
		config::GameObjectTemplatePermission {
			template: source.template as u16,
			rules: source.rules.into_iter().map(config::GroupsPermissionRule::from).collect(),
			fields: source.fields.into_iter().map(config::PermissionField::from).collect(),
		}
	}
}

impl From<proto::GroupsPermissionRule> for config::GroupsPermissionRule {
	fn from(source: proto::GroupsPermissionRule) -> Self {
		let deny = proto::PermissionLevel::Deny as i32;
		let ro = proto::PermissionLevel::Ro as i32;
		let rw = proto::PermissionLevel::Rw as i32;

		let permission = match source.permission {
			x if x == deny => config::Permission::Deny,
			x if x == ro => config::Permission::Ro,
			x if x == rw => config::Permission::Rw,
			_ => {
				panic!("Enum permission unrecognized {}", source.permission)
			}
		};
		config::GroupsPermissionRule {
			groups: AccessGroups(source.groups),
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

		let field_type = match source.r#type {
			x if x == event => crate::room::types::FieldType::Event,
			x if x == float => crate::room::types::FieldType::Float,
			x if x == long => crate::room::types::FieldType::Long,
			x if x == structure => crate::room::types::FieldType::Structure,
			_ => {
				panic!("Enum field_type unrecognized {}", source.r#type)
			}
		};
		config::PermissionField {
			id: source.id as u16,
			field_type,
			rules: source.rules.into_iter().map(config::GroupsPermissionRule::from).collect(),
		}
	}
}
