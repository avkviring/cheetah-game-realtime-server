use crate::constants::ClientId;
use serde::{Deserialize, Serialize};

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ClientOwner {
	Root,
	CurrentClient,
	Client(ClientId),
}

