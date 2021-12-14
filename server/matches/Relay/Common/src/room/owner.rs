use crate::room::RoomMemberId;

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum GameObjectOwner {
	Room,
	User(RoomMemberId),
}
