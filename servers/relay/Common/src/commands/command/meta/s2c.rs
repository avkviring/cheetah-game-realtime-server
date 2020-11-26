use serde::{Deserialize, Serialize};

use crate::commands::command::meta::c2s::C2SMetaCommandInformation;
use crate::room::UserPublicKey;

///
/// Служебная информация для исходящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S2CMetaCommandInformation {
	///
	/// Идентификатор клиента
	///
	pub user_public_key: UserPublicKey,
	
	///
	/// Условное время создание команды на клиенте
	///
	pub timestamp: u64,
}

impl S2CMetaCommandInformation {
	pub fn new(user_public_key: UserPublicKey, meta_from_client: &C2SMetaCommandInformation) -> Self {
		S2CMetaCommandInformation {
			user_public_key,
			timestamp: meta_from_client.timestamp,
		}
	}
}