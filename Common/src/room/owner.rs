use serde::{Deserialize, Serialize};

use crate::commands::hash::UserPublicKey;

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ClientOwner {
	Root,
	CurrentClient,
	Client(UserPublicKey),
}

