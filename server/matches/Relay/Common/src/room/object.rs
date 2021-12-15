use std::io::Cursor;

use crate::protocol::codec::cursor::VariableInt;
use crate::room::owner::GameObjectOwner;
use crate::room::RoomMemberId;

///
/// Идентификатор игрового объекта на клиенте
///
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct GameObjectId {
	///
	/// Создатель игрового объекта
	///
	pub owner: GameObjectOwner,

	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
}

impl GameObjectId {
	///
	/// Идентификатор первого клиентского объекта (для исключения пересечений с объектами клиента из конфигурации)
	///
	pub const CLIENT_OBJECT_ID_OFFSET: u32 = 512;

	pub fn new(id: u32, owner: GameObjectOwner) -> Self {
		GameObjectId { owner, id }
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.id as u64)?;
		match self.owner {
			GameObjectOwner::Room => out.write_variable_i64(-1),
			GameObjectOwner::User(user) => out.write_variable_i64(user as i64),
		}
	}
	pub fn decode(input: &mut Cursor<&mut [u8]>) -> std::io::Result<Self> {
		Ok(GameObjectId {
			id: input.read_variable_u64()? as u32,
			owner: match input.read_variable_i64()? {
				-1 => GameObjectOwner::Room,
				user_id => GameObjectOwner::User(user_id as RoomMemberId),
			},
		})
	}
}

impl Default for GameObjectId {
	fn default() -> Self {
		GameObjectId::new(0, GameObjectOwner::Room)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_encode_decode_room_owner() {
		let mut buffer = [0 as u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		let original = GameObjectId::new(100, GameObjectOwner::Room);
		original.encode(&mut cursor).unwrap();
		cursor.set_position(0);
		let actual = GameObjectId::decode(&mut cursor).unwrap();
		assert_eq!(original, actual);
	}

	#[test]
	fn should_encode_decode_user_owner() {
		let mut buffer = [0 as u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		let original = GameObjectId::new(100, GameObjectOwner::User(5));
		original.encode(&mut cursor).unwrap();
		cursor.set_position(0);
		let actual = GameObjectId::decode(&mut cursor).unwrap();
		assert_eq!(original, actual);
	}
}
