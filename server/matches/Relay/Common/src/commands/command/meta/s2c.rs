use serde::{Deserialize, Serialize};

use crate::commands::command::meta::c2s::C2SMetaCommandInformation;
use crate::room::object::GameObjectId;
use crate::room::UserId;

///
/// Служебная информация для исходящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S2CMetaCommandInformation {
	///
	/// Идентификатор клиента
	///
	pub user_id: UserId,

	///
	/// Условное время создание команды на клиенте
	///
	pub timestamp: u64,

	///
	/// Объект - источник команды
	///
	pub source_object: Option<GameObjectId>,
}

impl S2CMetaCommandInformation {
	pub fn new(user_id: UserId, meta_from_client: &C2SMetaCommandInformation) -> Self {
		S2CMetaCommandInformation {
			user_id,
			timestamp: meta_from_client.timestamp,
			source_object: meta_from_client.source_object.clone(),
		}
	}
}
