use serde::{Deserialize, Serialize};

///
/// Тип данных поля
///
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum FieldType {
	Long,
	Double,
	Structure,
	Event,
}
