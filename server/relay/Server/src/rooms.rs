use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_relay_common::protocol::others::user_id::UserAndRoomId;
use cheetah_relay_common::room::RoomId;

use crate::room::debug::tracer::CommandTracer;
use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::room::{Room, RoomRegisterUserError, RoomUserListener};

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	changed_rooms: HashSet<RoomId, FnvBuildHasher>,
	tracer: Rc<CommandTracer>,
}

#[derive(Debug)]
pub struct OutFrame {
	pub user_and_room_id: UserAndRoomId,
	pub frame: Frame,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
	RoomError(RoomRegisterUserError),
}

#[derive(Debug)]
pub enum RegisterRoomError {
	AlreadyRegistered,
}

impl Rooms {
	pub fn new(tracer: CommandTracer) -> Self {
		Self {
			room_by_id: Default::default(),
			changed_rooms: Default::default(),
			tracer: Rc::new(tracer),
		}
	}

	pub fn create_room(&mut self, template: RoomTemplate, listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> Result<(), RegisterRoomError> {
		let room_id = template.id.clone();
		if self.room_by_id.contains_key(&room_id) {
			Result::Err(RegisterRoomError::AlreadyRegistered)
		} else {
			let room = Room::new(template.clone(), self.tracer.clone(), listeners);
			self.room_by_id.insert(room_id, room);
			Result::Ok(())
		}
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<(), RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				room.register_user(template).map_err(|e| RegisterUserError::RoomError(e))?;
				Result::Ok(())
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

	pub fn on_frame_received(&mut self, user_and_room_id: UserAndRoomId, frame: Frame, now: &Instant) {
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
			changed_rooms: Default::default(),
			tracer: Rc::new(CommandTracer::new_with_allow_all()),
		}
	}
}
