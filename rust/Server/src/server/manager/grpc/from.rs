use crate::server::manager::grpc;
use crate::server::manager::grpc::proto;
use crate::server::manager::grpc::proto::field_value::Variant;
use crate::server::manager::grpc::proto::{GameObjectConfig, GameObjectTemplate, ItemConfig, Member, MemberStatus};
use crate::server::room::config::{member, object, room};
use crate::server::room::member::{RoomMember, RoomMemberStatus};
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectTemplateId;

impl From<grpc::RoomTemplate> for room::RoomCreateParams {
	fn from(source: grpc::RoomTemplate) -> room::RoomCreateParams {
		Self {
			name: source.template_name,
			objects: source.objects.into_iter().map(From::from).collect(),
			configs: source.configs.into_iter().map(|config| (config.template as GameObjectTemplateId, From::from(config))).collect(),
		}
	}
}

impl From<proto::GameObjectConfig> for object::GameObjectConfig {
	fn from(source: GameObjectConfig) -> Self {
		Self {
			items_config: source.items_config.into_iter().map(|item| (item.0 as FieldId, From::from(item.1))).collect(),
		}
	}
}

impl From<proto::ItemConfig> for object::ItemConfig {
	fn from(source: ItemConfig) -> Self {
		Self { capacity: source.capacity as usize }
	}
}

impl From<proto::UserTemplate> for member::MemberCreateParams {
	fn from(source: proto::UserTemplate) -> Self {
		member::MemberCreateParams::new_member(AccessGroups(source.groups), source.objects.into_iter().map(object::GameObjectCreateParams::from).collect())
	}
}

impl From<GameObjectTemplate> for object::GameObjectCreateParams {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: proto::GameObjectTemplate) -> Self {
		let fields: Vec<_> = source
			.fields
			.into_iter()
			.map(|f| {
				let value = f.value.unwrap();
				let field_id = f.id as FieldId;
				(field_id, value.variant.unwrap())
			})
			.collect();

		object::GameObjectCreateParams {
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

impl From<RoomMember> for Member {
	fn from(value: RoomMember) -> Self {
		Member {
			id: value.id,
			status: MemberStatus::from(value.status).into(),
		}
	}
}

impl From<RoomMemberStatus> for MemberStatus {
	fn from(value: RoomMemberStatus) -> Self {
		match value {
			RoomMemberStatus::Created => MemberStatus::Created,
			RoomMemberStatus::CreatedNotConnectedAndDeleted => MemberStatus::CreatedNotConnectedAndDeleted,
			RoomMemberStatus::Connected => MemberStatus::Connected,
			RoomMemberStatus::Attached => MemberStatus::Attached,
			RoomMemberStatus::Detached => MemberStatus::Detached,
			RoomMemberStatus::Disconnected => MemberStatus::Disconnected,
		}
	}
}
