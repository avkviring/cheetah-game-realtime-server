use crate::room::RoomMemberId;
use serde::{Deserialize, Serialize};

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum GameObjectOwner {
	Room,
	User(RoomMemberId),
}
