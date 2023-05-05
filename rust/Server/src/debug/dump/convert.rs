use cheetah_common::room::owner::GameObjectOwner;

use crate::debug::proto::admin;
use crate::debug::proto::shared::field_value;
use crate::debug::proto::shared::FieldValue;
use crate::debug::proto::shared::GameObjectField;
use crate::room::member::RoomMember;
use crate::room::object::GameObject;
use crate::room::Room;

impl From<&Room> for admin::DumpResponse {
	fn from(room: &Room) -> Self {
		let users = room.members.values().map(admin::DumpUser::from).collect();
		let objects = room.objects.iter().map(|(_k, o)| admin::DumpObject::from(o)).collect();
		Self { users, objects }
	}
}

impl From<&GameObject> for admin::DumpObject {
	fn from(source: &GameObject) -> Self {
		let fields = source
			.longs
			.get_fields()
			.map(|(id, v)| GameObjectField {
				id: u32::from(*id),
				value: Some(FieldValue {
					variant: Some(field_value::Variant::Long(*v)),
				}),
			})
			.chain(source.doubles.get_fields().map(|(id, v)| GameObjectField {
				id: u32::from(*id),
				value: Some(FieldValue {
					variant: Some(field_value::Variant::Double(*v)),
				}),
			}))
			.chain(source.structures.get_fields().map(|(id, v)| GameObjectField {
				id: u32::from(*id),
				value: Some(FieldValue {
					variant: Some(field_value::Variant::Structure(v.as_slice().to_vec())),
				}),
			}))
			.collect();

		Self {
			owner_user_id: match &source.id.get_owner() {
				GameObjectOwner::Room => None,
				GameObjectOwner::Member(id) => Some(u32::from(*id)),
			},
			id: source.id.id,
			template: u32::from(source.template_id),
			groups: source.access_groups.0,
			created: source.created,
			fields,
		}
	}
}

impl From<&RoomMember> for admin::DumpUser {
	fn from(member: &RoomMember) -> Self {
		Self {
			id: u32::from(member.id),
			groups: member.template.groups.0,
			attached: member.attached,
		}
	}
}
