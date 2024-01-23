use crate::server::room::command::ServerCommandError;
use crate::server::room::config::member::MemberCreateParams;
use crate::server::room::config::room::RoomCreateParams;
use crate::server::room::Room;
use cheetah_common::commands::{CommandWithChannelType, CommandWithReliabilityGuarantees};
use cheetah_game_realtime_protocol::others::member_id::MemberAndRoomId;
use cheetah_game_realtime_protocol::{RoomId, RoomMemberId};
use fnv::FnvBuildHasher;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Default)]
pub struct Rooms {
	rooms: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	pub created_rooms_count: usize,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("room not found {0}")]
pub struct RoomNotFoundError(pub RoomId);

impl Rooms {
	pub fn new() -> Self {
		Self {
			rooms: Default::default(),
			room_id_generator: 0,
			created_rooms_count: 0,
		}
	}

	pub(crate) fn get(&self, room_id: &RoomId) -> Option<&Room> {
		self.rooms.get(room_id)
	}
	pub(crate) fn rooms(&self) -> Iter<'_, RoomId, Room> {
		self.rooms.iter()
	}

	pub fn create_room(&mut self, template: RoomCreateParams) -> RoomId {
		self.room_id_generator += 1;
		self.created_rooms_count += 1;
		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template);
		self.rooms.insert(room_id, room);
		room_id
	}

	/// удалить комнату из списка без изменений пользователей и объектов
	pub fn force_remove_room(&mut self, room_id: &RoomId) -> Result<Room, RoomNotFoundError> {
		self.rooms.remove(room_id).ok_or(RoomNotFoundError(*room_id))
	}

	pub fn register_member(&mut self, room_id: RoomId, member_template: MemberCreateParams) -> Result<RoomMemberId, RoomNotFoundError> {
		match self.rooms.get_mut(&room_id) {
			None => Err(RoomNotFoundError(room_id)),
			Some(room) => Ok(room.register_member(member_template)),
		}
	}

	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomId, &RoomMemberId, &[CommandWithChannelType]),
	{
		for (room_id, room) in &mut self.rooms {
			room.collect_out_commands(|member_id, commands| {
				collector(room_id, member_id, commands);
			});
		}
	}

	pub fn execute_commands(&mut self, member_and_room_id: MemberAndRoomId, commands: &[CommandWithReliabilityGuarantees]) {
		match self.rooms.get_mut(&member_and_room_id.room_id) {
			None => {
				tracing::error!("[rooms] on_frame_received room({}) not found", member_and_room_id.room_id);
			}
			Some(room) => {
				room.execute_commands(member_and_room_id.member_id, commands);
			}
		}
	}

	pub fn member_disconnected(&mut self, member_and_room_id: &MemberAndRoomId) -> Result<(), ServerCommandError> {
		match self.rooms.get_mut(&member_and_room_id.room_id) {
			None => Err(ServerCommandError::RoomNotFound(RoomNotFoundError(member_and_room_id.room_id))),
			Some(room) => {
				room.disconnect_member(member_and_room_id.member_id)?;
				Ok(())
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_remove_room() {
		let mut rooms = Rooms::default();
		let room_id = rooms.create_room(RoomCreateParams::default());
		let room = rooms.force_remove_room(&room_id);
		assert!(room.is_ok(), "want room when take by room_id");
		assert_eq!(room_id, room.unwrap().id, "want taken room_id to match with room_id parameter");
		assert!(!rooms.rooms.contains_key(&room_id), "want room_id to be removed from rooms");
	}

	#[test]
	fn should_created_rooms_count() {
		let mut rooms = Rooms::default();
		let room_a = rooms.create_room(RoomCreateParams::default());
		rooms.create_room(RoomCreateParams::default());
		assert_eq!(rooms.created_rooms_count, 2);
		rooms.force_remove_room(&room_a).unwrap();
		assert_eq!(rooms.created_rooms_count, 2);
	}

	#[test]
	fn should_remove_room_room_not_found() {
		let mut rooms = Rooms::default();
		let room_id = 123;
		let room = rooms.force_remove_room(&room_id);
		assert!(room.is_err(), "want error when take non existing room");
		assert_eq!(room_id, room.err().unwrap().0, "want the same room_id in take_room parameter and error");
	}
}
