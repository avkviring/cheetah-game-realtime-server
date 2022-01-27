use hash32::Hasher;

use crate::room::RoomMemberId;

///
/// владелец - клиент или root
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum GameObjectOwner {
	Room,
	User(RoomMemberId),
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
			GameObjectOwner::User(user) => {
				hash32::Hash::hash(&1, state);
				hash32::Hash::hash(user, state);
			}
		}
	}
}
