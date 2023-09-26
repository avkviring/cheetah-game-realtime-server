use crate::server::manager::grpc::proto::internal;
use crate::server::manager::grpc::proto::shared::field_value::Variant;
use crate::server::room::template::config;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;

impl From<internal::RoomTemplate> for config::RoomTemplate {
	fn from(source: internal::RoomTemplate) -> config::RoomTemplate {
		config::RoomTemplate {
			name: source.template_name,
			objects: source.objects.into_iter().map(config::GameObjectTemplate::from).collect(),
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
