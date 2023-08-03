use crate::room::owner::GameObjectOwner;
use cheetah_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use cheetah_protocol::RoomMemberId;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Error, ErrorKind};

pub type GameObjectTemplateId = u16;

///
/// Идентификатор игрового объекта на клиенте
///
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct GameObjectId {
	///
	/// Идентификатор игрового объекта в рамках владельца
	///
	pub id: u32,
	pub is_room_owner: bool,
	pub member_id: RoomMemberId,
}

impl GameObjectId {
	///
	/// Идентификатор первого клиентского объекта (для исключения пересечений с объектами клиента из конфигурации)
	///
	pub const CLIENT_OBJECT_ID_OFFSET: u32 = 512;

	#[must_use]
	pub fn new(id: u32, owner: GameObjectOwner) -> Self {
		match owner {
			GameObjectOwner::Room => GameObjectId {
				id,
				is_room_owner: true,
				member_id: 0,
			},
			GameObjectOwner::Member(member_id) => GameObjectId { id, is_room_owner: false, member_id },
		}
	}

	#[must_use]
	pub fn is_owner(&self, other_member: RoomMemberId) -> bool {
		!self.is_room_owner && self.member_id == other_member
	}

	pub fn get_owner(&self) -> GameObjectOwner {
		if self.is_room_owner {
			GameObjectOwner::Room
		} else {
			GameObjectOwner::Member(self.member_id)
		}
	}

	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(u64::from(self.id))?;
		if self.is_room_owner {
			out.write_variable_i64(-1)
		} else {
			out.write_variable_i64(i64::from(self.member_id))
		}
	}
	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let id = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
		let owner = input.read_variable_i64()?;
		let room_owner = owner == -1;
		let member_id = if room_owner { 0 } else { owner.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))? };
		Ok(GameObjectId {
			id,
			is_room_owner: room_owner,
			member_id,
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
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		let original = GameObjectId::new(100, GameObjectOwner::Room);
		original.encode(&mut cursor).unwrap();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = GameObjectId::decode(&mut read_cursor).unwrap();
		assert_eq!(original, actual);
	}

	#[test]
	fn should_encode_decode_member_owner() {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		let original = GameObjectId::new(100, GameObjectOwner::Member(5));
		original.encode(&mut cursor).unwrap();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = GameObjectId::decode(&mut read_cursor).unwrap();
		assert_eq!(original, actual);
	}
}
