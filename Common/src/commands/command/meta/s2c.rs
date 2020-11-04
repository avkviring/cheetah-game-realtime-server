use serde::{Deserialize, Serialize};

use crate::commands::command::meta::c2s::C2SMetaCommandInformation;
use crate::constants::ClientId;

///
/// Служебная информация для исходящей команды
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S2CMetaCommandInformation {
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
    pub fn new(client: ClientId, meta_from_client: &C2SMetaCommandInformation) -> Self {
        S2CMetaCommandInformation {
            client,
            timestamp: meta_from_client.timestamp,
        }
    }
}