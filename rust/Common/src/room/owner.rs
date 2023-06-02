use cheetah_protocol::RoomMemberId;

///
/// владелец - клиент или root
///
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum GameObjectOwner {
	Room,
	Member(RoomMemberId),
}
