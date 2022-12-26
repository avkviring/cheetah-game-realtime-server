use std::collections::hash_map::Iter;

use fnv::FnvHashMap;

use cheetah_common::commands::field::{Field, FieldId};
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::FieldType;
use cheetah_common::room::RoomMemberId;

use crate::room::object::CreateCommandsCollector;

#[derive(Debug, Default, Clone)]
pub struct Fields<T> {
	values: FnvHashMap<FieldId, T>,
}

impl<T> Fields<T>
where
	T: Copy,
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
	pub(crate) fn collect_commands<F>(&self, out_commands: &mut CreateCommandsCollector, member_id: RoomMemberId, factory: F) -> Result<(), S2CCommandWithMeta>
	where
		F: Fn(FieldId, T) -> S2CCommand,
	{
		for (field_id, value) in self.values.iter() {
			let command = S2CCommandWithMeta {
				field: Some(Field {
					id: *field_id,
					field_type: FieldType::Long,
				}),
				creator: member_id,
				command: factory(*field_id, *value),
			};
			out_commands.push(command)?;
		}
		Ok(())
	}

	pub(crate) fn get_fields(&self) -> Iter<'_, FieldId, T> {
		self.values.iter()
	}
}
