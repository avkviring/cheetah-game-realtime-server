use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::server::room::object::S2CCommandsCollector;

pub mod f64;
pub mod i64;
pub mod structure;

pub mod vec;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Fields<T>
where
	T: FieldValue,
{
	values: FnvHashMap<FieldId, T>,
	collector: Vec<S2CCommand>,
}

impl<T> Fields<T>
where
	T: FieldValue,
{
	pub(crate) fn set(&mut self, field_id: FieldId, value: T) {
		self.values.insert(field_id, value);
	}
	pub(crate) fn get(&self, field_id: FieldId) -> Option<&T> {
		self.values.get(&field_id)
	}

	pub(crate) fn get_mut(&mut self, field_id: FieldId) -> Option<&mut T> {
		self.values.get_mut(&field_id)
	}

	pub(crate) fn delete(&mut self, field_id: FieldId) {
		self.values.remove(&field_id);
	}
	pub(crate) fn collect_commands(&mut self, out_commands: &mut S2CCommandsCollector, object_id: GameObjectId) {
		for (field_id, value) in self.values.iter() {
			self.collector.clear();
			value.into(object_id, *field_id, &mut self.collector);
			self.collector.iter().for_each(|command| {
				out_commands.push(command.clone());
			});
		}
	}
}

pub trait FieldValue {
	fn into(&self, object_id: GameObjectId, field_id: FieldId, collector: &mut Vec<S2CCommand>);
}
