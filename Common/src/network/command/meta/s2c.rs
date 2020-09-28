use serde::{Deserialize, Serialize};

use crate::constants::ClientId;
use crate::network::command::meta::c2s::C2SMetaCommandInformation;

///
/// Служебная информация для исходящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S2CMetaCommandInformation {
	pub command_code: u8,
	
	///
	/// Идентификатор клиента
	///
	pub client: ClientId,
	
	///
	/// Условное время создание команды на клиенте
	///
	pub timestamp: u64,
}

impl S2CMetaCommandInformation {
	pub fn new(
		command_code: u8,
		client: ClientId,
		meta_from_client: &C2SMetaCommandInformation)
		-> Self {
		S2CMetaCommandInformation {
			command_code,
			client,
			timestamp: meta_from_client.timestamp,
		}
	}
}