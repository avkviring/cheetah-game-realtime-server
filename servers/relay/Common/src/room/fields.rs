use std::collections::HashMap;
use std::ptr;

use fnv::{FnvBuildHasher, FnvHashMap};
use serde::{Deserialize, Serialize};

use crate::constants::FieldID;

pub type HeaplessBuffer = heapless::Vec<u8, heapless::consts::U256>;
pub type HeaplessLongMap = heapless::FnvIndexMap<FieldID, i64, heapless::consts::U256>;
pub type HeapLessFloatMap = heapless::FnvIndexMap<FieldID, f64, heapless::consts::U256>;

///
/// Описание данных полей игрового объекта
///
#[derive(Debug, Serialize, Deserialize)]
pub struct GameObjectFields {
	pub longs: HeaplessLongMap,
	pub floats: HeapLessFloatMap,
	pub structures: HashMap<FieldID, HeaplessBuffer, FnvBuildHasher>,
}

impl Clone for GameObjectFields {
	fn clone(&self) -> Self {
		let mut result = Self::default();
		unsafe {
			ptr::copy_nonoverlapping(&self.longs, &mut result.longs, 1);
			ptr::copy_nonoverlapping(&self.floats, &mut result.floats, 1);
		}
		result.structures.clone_from(&self.structures);
		result
	}
}

impl PartialEq for GameObjectFields {
	fn eq(&self, other: &Self) -> bool {
		self.structures.eq(&other.structures)
			&& self.longs.eq(&other.longs)
			&& self
				.floats
				.iter()
				.find(|(key, value)| {
					if let Some(other_value) = other.floats.get(key) {
						(*other_value - **value).abs() > 0.0000001
					} else {
						true
					}
				})
				.is_none()
	}
}

impl Default for GameObjectFields {
	fn default() -> Self {
		GameObjectFields {
			longs: Default::default(),
			floats: Default::default(),
			structures: FnvHashMap::with_capacity_and_hasher(256, Default::default()),
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::room::fields::GameObjectFields;

	#[test]
	pub fn test_clone() {
		let mut source = GameObjectFields::default();
		source.longs.insert(1, 100).unwrap();
		source.floats.insert(2, 200.200).unwrap();
		source.structures.insert(3, create_structure());

		let dest = source.clone();
		assert_eq!(source, dest);
	}

	#[test]
	pub fn test_eq() {
		let mut fields_a = GameObjectFields::default();
		fields_a.longs.insert(1, 100).unwrap();
		fields_a.floats.insert(2, 200.200).unwrap();
		fields_a.structures.insert(3, create_structure());

		let mut fields_b = GameObjectFields::default();
		fields_b.longs.insert(1, 100).unwrap();
		assert_ne!(fields_a, fields_b);
		fields_b.floats.insert(2, 200.200).unwrap();
		assert_ne!(fields_a, fields_b);
		fields_b.structures.insert(3, create_structure());
		assert_eq!(fields_a, fields_b);
	}

	fn create_structure() -> heapless::Vec<u8, heapless::consts::U256> {
		let mut result = heapless::Vec::new();
		result.extend_from_slice(vec![1, 2, 3].as_slice()).unwrap();
		result
	}
}
