use serde::{Deserialize, Serialize};

use crate::room::object::GameObjectId;

///
/// Служебная информация для каждой входящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct C2SMetaCommandInformation {
	///
	/// Условное время создания команды на клиенте
	///
	pub timestamp: u64,
	///
	/// Объект - источник команды
	///
	pub source_object: Option<GameObjectId>,
}
