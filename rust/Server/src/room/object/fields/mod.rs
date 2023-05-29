use std::collections::hash_map::Iter;

use fnv::FnvHashMap;

use cheetah_common::commands::field::{Field, FieldId};
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::FieldType;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::room::object::S2CCommandsCollector;

pub mod f64;
pub mod i64;
pub mod structure;

#[derive(Debug, Default, Clone)]
pub struct Fields<T>
where
	T: FieldValue,
{
	values: FnvHashMap<FieldId, T>,
}

impl<T> Fields<T>
where
	T: FieldValue,
{
	pub fn set(&mut self, field_id: FieldId, value: T) {
		self.values.insert(field_id, value);
	}
	pub(crate) fn get(&self, field_id: FieldId) -> Option<&T> {
		self.values.get(&field_id)
	}

	pub(crate) fn delete(&mut self, field_id: FieldId) {
		self.values.remove(&field_id);
	}
	pub(crate) fn collect_commands(&self, out_commands: &mut S2CCommandsCollector, member_id: RoomMemberId, object_id: GameObjectId) {
		for (field_id, value) in self.values.iter() {
			let command = value.into(object_id.clone(), field_id.clone());
			let s2c_command_with_meta = S2CCommandWithMeta {
				field: Some(Field {
					id: *field_id,
					field_type: FieldType::Long,
				}),
				creator: member_id,
				command: command.clone(),
			};
			out_commands.push(s2c_command_with_meta);
		}
	}

	pub(crate) fn get_fields(&self) -> Iter<'_, FieldId, T> {
		self.values.iter()
	}
}

pub trait FieldValue {
	fn into(&self, object_id: GameObjectId, field_id: FieldId) -> S2CCommand;
}