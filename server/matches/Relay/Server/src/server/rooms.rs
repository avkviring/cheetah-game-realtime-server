use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::room::{Room, RoomUserListener};

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: HashSet<RoomId, FnvBuildHasher>,
}

#[derive(Debug)]
pub struct OutFrame {
	pub user_and_room_id: MemberAndRoomId,
	pub frame: Frame,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
}

impl Rooms {
	pub fn new() -> Self {
		Self {
			room_by_id: Default::default(),
			room_id_generator: 0,
			changed_rooms: Default::default(),
		}
	}

	pub fn create_room(&mut self, template: RoomTemplate, listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> RoomId {
		self.room_id_generator += 1;
		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template, listeners);
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => Result::Ok(room.register_user(template)),
		}
	}

	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		let mut data: [FrameId; 30_000] = [0; 30_000];
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
					room.collect_out_frame(out_frames, now);
				}
			}
		}
	}

	pub fn on_frame_received(&mut self, user_and_room_id: MemberAndRoomId, frame: Frame, now: &Instant) {
		match self.room_by_id.get_mut(&user_and_room_id.room_id) {
			None => {
				log::error!("[rooms] on_frame_received room({}) not found", user_and_room_id.room_id);
			}
			Some(room) => {
				room.process_in_frame(user_and_room_id.user_id, frame, now);
				self.changed_rooms.insert(room.id);
			}
		}
	}

	pub fn cycle(&mut self, now: &Instant) {
		let mut changed = [0; 30_000];
		let mut index = 0;
		self.room_by_id.values_mut().for_each(|room| {
			if room.cycle(now) {
				changed[index] = room.id;
				index += 1;
			}
		});
		for i in 0..index {
			self.changed_rooms.insert(changed[i]);
		}
	}
}

#[cfg(test)]
impl Default for Rooms {
	fn default() -> Self {
		Self {
			room_by_id: Default::default(),
			room_id_generator: 0,
			changed_rooms: Default::default(),
		}
	}
}
