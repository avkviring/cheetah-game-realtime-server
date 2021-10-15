use std::collections::HashMap;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::owner::ObjectOwner;

use crate::debug::proto::admin;
use crate::room::object::GameObject;
use crate::room::{Room, User};

impl From<&Room> for admin::DumpResponse {
	fn from(room: &Room) -> Self {
		let users = room.users.iter().map(|(k, u)| admin::DumpUser::from(u)).collect();
		let objects = room.objects.iter().map(|(k, o)| admin::DumpObject::from(o)).collect();
		Self { users, objects }
	}
}

impl From<&GameObject> for admin::DumpObject {
	fn from(source: &GameObject) -> Self {
		Self {
			user: match &source.id.owner {
				ObjectOwner::Root => u32::MAX,
				ObjectOwner::User(id) => *id as u32,
			},
			id: source.id.id,
			template: source.template as u32,
			groups: source.access_groups.0,
			created: source.created,
			longs: from(&source.longs),
			floats: from(&source.floats),
			compare_and_set_owners: from(&source.compare_and_set_owners),
			structures: from(&source.structures),
		}
	}
}

fn from<IN: Clone, OUT: From<IN>>(source: &HashMap<FieldId, IN, FnvBuildHasher>) -> HashMap<u32, OUT> {
	let cloned = (*source).clone();
	cloned.into_iter().map(|(k, v)| (k as u32, OUT::from(v))).collect()
}

impl From<&User> for admin::DumpUser {
	fn from(user: &User) -> Self {
		Self {
			id: user.id as u32,
			groups: user.template.groups.0,
			attached: user.attached,
		}
	}
}
