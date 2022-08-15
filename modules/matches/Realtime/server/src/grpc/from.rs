use cheetah_matches_realtime_common::{commands::FieldValue, room::access::AccessGroups};

use crate::debug::proto::shared::{
	field_value::Variant as VariantDebug, FieldValue as GRPCFieldValueDebug,
};
use crate::grpc::proto::internal;
use crate::grpc::proto::shared::{self, field_value::Variant, FieldValue as GRPCFieldValue};
use crate::room::object::Field;
use crate::room::template::config;

impl From<internal::RoomTemplate> for config::RoomTemplate {
	fn from(source: internal::RoomTemplate) -> config::RoomTemplate {
		config::RoomTemplate {
			name: source.template_name,
			objects: source
				.objects
				.into_iter()
				.map(config::GameObjectTemplate::from)
				.collect(),
			permissions: config::Permissions::from(source.permissions.unwrap_or_default()),
		}
	}
}

impl From<internal::UserTemplate> for config::MemberTemplate {
	fn from(source: internal::UserTemplate) -> Self {
		config::MemberTemplate::new_member(
			AccessGroups(source.groups),
			source
				.objects
				.into_iter()
				.map(config::GameObjectTemplate::from)
				.collect(),
		)
	}
}

impl From<internal::GameObjectTemplate> for config::GameObjectTemplate {
	fn from(source: internal::GameObjectTemplate) -> Self {
		config::GameObjectTemplate {
			id: source.id,
			template: source.template as u16,
			groups: AccessGroups(source.groups),
			fields: source
				.fields
				.into_iter()
				.map(|f| {
					let field_value: FieldValue = f.value.expect("Field with no value").into();
					((f.id as u16, field_value.field_type()), field_value)
				})
				.collect(),
		}
	}
}

impl From<GRPCFieldValue> for FieldValue {
	fn from(field: GRPCFieldValue) -> Self {
		let variant = field.variant.expect("FieldValue was empty");
		match variant {
			Variant::Double(v) => FieldValue::Double(v),
			Variant::Long(v) => FieldValue::Long(v),
			Variant::Structure(s) => FieldValue::Structure(s),
		}
	}
}

impl From<FieldValue> for GRPCFieldValueDebug {
	fn from(value: FieldValue) -> Self {
		let value_d = match value {
			FieldValue::Double(v) => VariantDebug::Double(v),
			FieldValue::Long(v) => VariantDebug::Long(v),
			FieldValue::Structure(s) => VariantDebug::Structure(s),
		};

		GRPCFieldValueDebug {
			variant: Some(value_d),
		}
	}
}

impl From<internal::Permissions> for config::Permissions {
	fn from(source: internal::Permissions) -> Self {
		config::Permissions {
			templates: source
				.objects
				.into_iter()
				.map(config::GameObjectTemplatePermission::from)
				.collect(),
		}
	}
}

impl From<internal::GameObjectTemplatePermission> for config::GameObjectTemplatePermission {
	fn from(source: internal::GameObjectTemplatePermission) -> Self {
		config::GameObjectTemplatePermission {
			template: source.template as u16,
			rules: source
				.rules
				.into_iter()
				.map(config::GroupsPermissionRule::from)
				.collect(),
			fields: source
				.fields
				.into_iter()
				.map(config::PermissionField::from)
				.collect(),
		}
	}
}

impl From<internal::GroupsPermissionRule> for config::GroupsPermissionRule {
	fn from(source: internal::GroupsPermissionRule) -> Self {
		let deny = internal::PermissionLevel::Deny as i32;
		let ro = internal::PermissionLevel::Ro as i32;
		let rw = internal::PermissionLevel::Rw as i32;

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

impl From<internal::PermissionField> for config::PermissionField {
	fn from(source: internal::PermissionField) -> Self {
		let event = shared::FieldType::Event as i32;
		let double = shared::FieldType::Double as i32;
		let long = shared::FieldType::Long as i32;
		let structure = shared::FieldType::Structure as i32;

		let field_type = match source.r#type {
			x if x == event => cheetah_matches_realtime_common::commands::FieldType::Event,
			x if x == double => cheetah_matches_realtime_common::commands::FieldType::Double,
			x if x == long => cheetah_matches_realtime_common::commands::FieldType::Long,
			x if x == structure => cheetah_matches_realtime_common::commands::FieldType::Structure,
			_ => {
				panic!("Enum field_type unrecognized {}", source.r#type)
			}
		};
		config::PermissionField {
			field: Field {
				id: source.id as u16,
				field_type,
			},
			rules: source
				.rules
				.into_iter()
				.map(config::GroupsPermissionRule::from)
				.collect(),
		}
	}
}
