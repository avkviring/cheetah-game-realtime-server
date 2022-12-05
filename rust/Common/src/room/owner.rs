use hash32::Hasher;

use crate::room::RoomMemberId;

///
/// владелец - клиент или root
///
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum GameObjectOwner {
	Room,
	Member(RoomMemberId),
}

impl hash32::Hash for GameObjectOwner {
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		match self {
			GameObjectOwner::Room => {
				hash32::Hash::hash(&0, state);
			}
			GameObjectOwner::Member(member_id) => {
				hash32::Hash::hash(&1, state);
				hash32::Hash::hash(member_id, state);
			}
		}
	}
}
