use crate::room::RoomMemberId;

///
/// владелец - клиент или root
///
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum GameObjectOwner {
	Room,
	Member(RoomMemberId),
}
