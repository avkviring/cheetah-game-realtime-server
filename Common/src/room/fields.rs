use std::collections::HashMap;

use fnv::{FnvBuildHasher, FnvHashMap};
use serde::{Deserialize, Serialize};

use crate::constants::FieldID;

///
/// Описание данных полей игрового объекта
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameObjectFields {
	/// счетчики
	pub longs: HashMap<FieldID, i64, FnvBuildHasher>,
	pub floats: HashMap<FieldID, f64, FnvBuildHasher>,
	/// структуры (для сервера это массивы данных)
	pub structures: HashMap<FieldID, Vec<u8>, FnvBuildHasher>,
}


impl Default for GameObjectFields {
	fn default() -> Self {
		GameObjectFields {
			longs: FnvHashMap::default(),
			floats: FnvHashMap::default(),
			structures: FnvHashMap::default(),
		}
	}
}