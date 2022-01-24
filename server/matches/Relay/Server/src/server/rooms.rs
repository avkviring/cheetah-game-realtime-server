use fnv::FnvBuildHasher;
use std::collections::{HashMap, HashSet, VecDeque};

use cheetah_matches_relay_common::protocol::commands::output::OutCommand;
use cheetah_matches_relay_common::protocol::frame::applications::CommandWithChannel;
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::room::Room;

#[derive(Default)]
pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: HashSet<RoomId, FnvBuildHasher>,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
}

impl Rooms {
	pub fn create_room(&mut self, template: RoomTemplate) -> RoomId {
		self.room_id_generator += 1;
		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template);
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => Result::Ok(room.register_user(template)),
		}
	}

	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomId, &RoomMemberId, &mut VecDeque<OutCommand>),
	{
		let mut data: [RoomId; 30_000] = [0; 30_000];
		let mut index = 0;
		self.changed_rooms.iter().for_each(|room_id| {
			data[index] = *room_id;
			index += 1;
		});
		self.changed_rooms.clear();

		for i in 0..index {
			match self.room_by_id.get_mut(&data[i]) {
				None => {}
				Some(room) => {
					let room_id = room.id;
					room.collect_out_commands(|user_id, commands| collector(&room_id, user_id, commands));
				}
			}
		}
	}

	pub fn execute_commands(&mut self, user_and_room_id: MemberAndRoomId, commands: &[CommandWithChannel]) {
		match self.room_by_id.get_mut(&user_and_room_id.room_id) {
			None => {
				log::error!("[rooms] on_frame_received room({}) not found", user_and_room_id.room_id);
			}
			Some(room) => {
				room.execute_commands(user_and_room_id.member_id, commands);
				self.changed_rooms.insert(room.id);
			}
		}
	}
	pub fn user_disconnected(&mut self, member_and_room_id: &MemberAndRoomId) {
		match self.room_by_id.get_mut(&member_and_room_id.room_id) {
			None => {
				log::error!("[rooms] room not found ({:?}) in user_disconnect", member_and_room_id);
			}
			Some(room) => {
				room.disconnect_user(member_and_room_id.member_id);
				self.changed_rooms.insert(member_and_room_id.room_id);
			}
		}
	}
}
