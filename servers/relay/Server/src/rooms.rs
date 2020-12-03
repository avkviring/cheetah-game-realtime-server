use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_relay_common::room::UserPublicKey;

use crate::room::template::{RoomTemplate, UserTemplate};
use crate::room::{Room, RoomId};

#[derive(Default)]
pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	pub user_to_room: HashMap<UserPublicKey, RoomId, FnvBuildHasher>,
	changed_rooms: HashSet<RoomId, FnvBuildHasher>,
}

#[derive(Debug)]
pub struct OutFrame {
	pub user_public_key: UserPublicKey,
	pub frame: Frame,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
	AlreadyRegistered,
}

#[derive(Debug)]
pub enum RegisterRoomError {
	AlreadyRegistered,
}

impl Rooms {
	pub fn create_room(&mut self, config: RoomTemplate) -> Result<(), RegisterRoomError> {
		let room_id = config.id.clone();
		if self.room_by_id.contains_key(&room_id) {
			Result::Err(RegisterRoomError::AlreadyRegistered)
		} else {
			let room = Room::new(config.clone());
			self.room_by_id.insert(room_id, room);
			config.users.iter().for_each(|config| {
				self.user_to_room.insert(config.public_key, room_id);
			});
			Result::Ok(())
		}
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<(), RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				let public_key = template.public_key;
				if !(self.user_to_room.contains_key(&public_key)) {
					room.register_user(template);
					self.user_to_room.insert(public_key, room.id);
					Result::Ok(())
				} else {
					Result::Err(RegisterUserError::AlreadyRegistered)
				}
			}
		}
	}

	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		let mut data: [FrameId; 30_000] = [0; 30_000];
		let mut index = 0;
		self.changed_rooms.iter().for_each(|room_id| {
			data[index] = room_id.clone();
			index += 1;
		});
		self.changed_rooms.clear();

		for i in 0..index {
			match self.room_by_id.get_mut(&data[i]) {
				None => {}
				Some(room) => {
					room.collect_out_frame(out_frames, now);
				}
			}
		}
	}

	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		match self.user_to_room.get(user_public_key) {
			None => {
				log::error!("[rooms] user({:?} not found ", user_public_key);
			}
			Some(room_id) => match self.room_by_id.get_mut(room_id) {
				None => {}
				Some(room) => {
					room.process_in_frame(user_public_key, frame, now);
					self.changed_rooms.insert(room.id);
				}
			},
		}
	}

	pub fn cycle(&mut self, now: &Instant) {
		self.room_by_id.values_mut().for_each(|room| room.cycle(now));
	}
}
