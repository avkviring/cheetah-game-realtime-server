use serde::{Deserialize, Serialize};

///
/// Служебная информация для каждой входящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct C2SMetaCommandInformation {
	///
	/// Условное время создания команды на клиенте
	///
	pub timestamp: u64,
}
