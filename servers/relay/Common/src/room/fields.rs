use crate::constants::FieldID;
use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type HeaplessBuffer = heapless::Vec<u8, heapless::consts::U256>;
///
/// Описание данных полей игрового объекта
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct GameObjectFields {
	pub longs: HashMap<FieldID, i64, FnvBuildHasher>,
	pub floats: HashMap<FieldID, f64, FnvBuildHasher>,
	pub structures: HashMap<FieldID, Vec<u8>, FnvBuildHasher>,
}

#[cfg(test)]
mod tests {
	use crate::room::fields::GameObjectFields;

	#[test]
	pub fn test_clone() {
		let mut source = GameObjectFields::default();
		source.longs.insert(1, 100);
		source.floats.insert(2, 200.200);
		source.structures.insert(3, vec![1, 2, 3]);

		let dest = source.clone();
		assert_eq!(source, dest);
	}

	#[test]
	pub fn test_eq() {
		let mut fields_a = GameObjectFields::default();
		fields_a.longs.insert(1, 100);
		fields_a.floats.insert(2, 200.200);
		fields_a.structures.insert(3, vec![1, 2, 3]);

		let mut fields_b = GameObjectFields::default();
		fields_b.longs.insert(1, 100);
		assert_ne!(fields_a, fields_b);
		fields_b.floats.insert(2, 200.200);
		assert_ne!(fields_a, fields_b);
		fields_b.structures.insert(3, vec![1, 2, 3]);
		assert_eq!(fields_a, fields_b);
	}
}
