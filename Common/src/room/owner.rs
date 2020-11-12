use serde::{Deserialize, Serialize};
use crate::room::UserPublicKey;


///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ClientOwner {
	Root,
	User(UserPublicKey),
}

