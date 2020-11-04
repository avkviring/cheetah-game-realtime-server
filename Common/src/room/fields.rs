use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::constants::FieldID;

///
/// Описание данных полей игрового объекта
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameObjectFields {
	/// счетчики
	pub longs: HashMap<FieldID, i64>,
	pub floats: HashMap<FieldID, f64>,
	/// структуры (для сервера это массивы данных)
	pub structures: HashMap<FieldID, Vec<u8>>,
}


impl Default for GameObjectFields {
	fn default() -> Self {
		GameObjectFields {
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
		}
	}
}