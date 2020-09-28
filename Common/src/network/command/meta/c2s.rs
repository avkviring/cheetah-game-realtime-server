use serde::{Deserialize, Serialize};

///
/// Служебная информация для каждой входящей команды
///
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct C2SMetaCommandInformation {
	pub command_code: u8,
	///
	/// Условное время создания команды на клиенте
	///
	pub timestamp: u64,
}


impl C2SMetaCommandInformation {
	pub fn new(command_code: u8, timestamp: u64) -> Self {
		Self {
			command_code,
			timestamp,
		}
	}
}