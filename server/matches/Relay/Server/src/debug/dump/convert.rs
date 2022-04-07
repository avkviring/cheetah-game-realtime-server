use std::collections::HashMap;

use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;

use crate::debug::proto::admin;
use crate::room::object::GameObject;
use crate::room::{Member, Room};

impl From<&Room> for admin::DumpResponse {
	fn from(room: &Room) -> Self {
		let users = room.members.iter().map(|(_k, u)| admin::DumpUser::from(u)).collect();
		let objects = room.objects.iter().map(|(_k, o)| admin::DumpObject::from(o)).collect();
		Self { users, objects }
	}
}

impl From<&GameObject> for admin::DumpObject {
	fn from(source: &GameObject) -> Self {
		Self {
			owner_user_id: match &source.id.owner {
				GameObjectOwner::Room => Option::None,
				GameObjectOwner::Member(id) => Option::Some(*id as u32),
			},
			id: source.id.id,
			template: source.template_id as u32,
			groups: source.access_groups.0,
			created: source.created,
			longs: from(source.get_longs()),
			floats: from(source.get_doubles()),
			compare_and_set_owners: from(source.get_compare_and_set_owners()),
			structures: from(source.get_structures()),
		}
	}
}

fn from<IN: Clone, OUT: From<IN>, const N: usize>(source: &heapless::FnvIndexMap<FieldId, IN, N>) -> HashMap<u32, OUT> {
	source.iter().map(|(k, v)| (*k as u32, OUT::from(v.clone()))).collect()
}

impl From<&Member> for admin::DumpUser {
	fn from(user: &Member) -> Self {
		Self {
			id: user.id as u32,
			groups: user.template.groups.0,
			attached: user.attached,
			compare_and_set_cleaners: user
				.compare_and_sets_cleaners
				.iter()
				.map(|((object_id, field_id), value)| admin::CompareAndSetsCleaners {
					game_object_id: object_id.id,
					game_object_owner_user: match object_id.owner {
						GameObjectOwner::Room => u32::MAX,
						GameObjectOwner::Member(id) => id as u32,
					},
					field_id: *field_id as u32,
					reset: *value,
				})
				.collect(),
		}
	}
}
