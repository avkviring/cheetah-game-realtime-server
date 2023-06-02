use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::{Field, FieldId, FieldType};

use crate::grpc::proto::internal;
use crate::grpc::proto::shared::{self, field_value::Variant};
use crate::room::template::config;

impl From<internal::RoomTemplate> for config::RoomTemplate {
	fn from(source: internal::RoomTemplate) -> config::RoomTemplate {
		config::RoomTemplate {
			name: source.template_name,
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
			permissions: config::Permissions::from(source.permissions.unwrap_or_default()),
		}
	}
}

impl From<internal::UserTemplate> for config::MemberTemplate {
	fn from(source: internal::UserTemplate) -> Self {
		config::MemberTemplate::new_member(AccessGroups(source.groups), source.objects.into_iter().map(config::GameObjectTemplate::from).collect())
	}
}

impl From<internal::GameObjectTemplate> for config::GameObjectTemplate {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: internal::GameObjectTemplate) -> Self {
		let fields: Vec<_> = source
			.fields
			.into_iter()
			.map(|f| {
				let value = f.value.unwrap();
				let field_id = f.id as FieldId;
				(field_id, value.variant.unwrap())
			})
			.collect();

		config::GameObjectTemplate {
			id: source.id,
			template: source.template as u16,
			groups: AccessGroups(source.groups),
			doubles: fields
				.iter()
				.map(|(field_id, value)| if let Variant::Double(v) = value { Some((*field_id, *v)) } else { None })
				.flatten()
				.collect(),
			structures: fields
				.iter()
				.map(|(field_id, value)| if let Variant::Structure(v) = value { Some((*field_id, Buffer::from(v.as_ref()))) } else { None })
				.flatten()
				.collect(),
			longs: fields
				.iter()
				.map(|(field_id, value)| if let Variant::Long(v) = value { Some((*field_id, *v)) } else { None })
				.flatten()
				.collect(),
		}
	}
}

impl From<internal::Permissions> for config::Permissions {
	fn from(source: internal::Permissions) -> Self {
		config::Permissions {
			templates: source.objects.into_iter().map(config::GameObjectTemplatePermission::from).collect(),
		}
	}
}

impl From<internal::GameObjectTemplatePermission> for config::GameObjectTemplatePermission {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: internal::GameObjectTemplatePermission) -> Self {
		config::GameObjectTemplatePermission {
			template: source.template as u16,
			rules: source.rules.into_iter().map(config::GroupsPermissionRule::from).collect(),
			fields: source.fields.into_iter().map(config::PermissionField::from).collect(),
		}
	}
}

impl From<internal::GroupsPermissionRule> for config::GroupsPermissionRule {
	fn from(source: internal::GroupsPermissionRule) -> Self {
		config::GroupsPermissionRule {
			groups: AccessGroups(source.groups),
			permission: num::FromPrimitive::from_i32(source.permission).expect("Enum permission unrecognized"),
		}
	}
}

impl From<internal::PermissionField> for config::PermissionField {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: internal::PermissionField) -> Self {
		let event = shared::FieldType::Event as i32;
		let double = shared::FieldType::Double as i32;
		let long = shared::FieldType::Long as i32;
		let structure = shared::FieldType::Structure as i32;

		let field_type = match source.r#type {
			x if x == event => FieldType::Event,
			x if x == double => FieldType::Double,
			x if x == long => FieldType::Long,
			x if x == structure => FieldType::Structure,
			_ => {
				panic!("Enum field_type unrecognized {}", source.r#type)
			}
		};
		config::PermissionField {
			field: Field { id: source.id as u16, field_type },
			rules: source.rules.into_iter().map(config::GroupsPermissionRule::from).collect(),
		}
	}
}
