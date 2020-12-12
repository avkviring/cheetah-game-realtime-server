use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_relay_common::protocol::frame::{Frame, FrameId};
use cheetah_relay_common::room::UserPublicKey;

use crate::room::template::{RoomTemplate, UserTemplate};
use crate::room::tracer::Tracer;
use crate::room::{Room, RoomId, RoomRegisterUserError, RoomUserListener};

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	pub users: Rc<RefCell<Users>>,
	changed_rooms: HashSet<RoomId, FnvBuildHasher>,
	tracer: Rc<Tracer>,
}

#[derive(Default)]
pub struct Users {
	pub users: HashMap<UserPublicKey, RoomId, FnvBuildHasher>,
}

#[derive(Debug)]
pub struct OutFrame {
	pub user_public_key: UserPublicKey,
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
	pub fn new(tracer: Tracer) -> Self {
		Self {
			room_by_id: Default::default(),
			users: Rc::new(RefCell::new(Default::default())),
			changed_rooms: Default::default(),
			tracer: Rc::new(tracer),
		}
	}

	pub fn create_room(&mut self, template: RoomTemplate, mut listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> Result<(), RegisterRoomError> {
		let room_id = template.id.clone();
		if self.room_by_id.contains_key(&room_id) {
			Result::Err(RegisterRoomError::AlreadyRegistered)
		} else {
			listeners.push(self.users.clone());
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

	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		let room_id = (*self.users.clone()).borrow_mut().users.get(user_public_key).cloned();
		match room_id {
			None => {
				log::error!("[rooms] user({:?} not found ", user_public_key);
			}
			Some(room_id) => match self.room_by_id.get_mut(&room_id) {
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

impl RoomUserListener for Users {
	fn register_user(&mut self, room_id: u64, template: &UserTemplate) {
		self.users.insert(template.public_key.clone(), room_id.clone());
	}

	fn connected_user(&mut self, _: u64, _: &UserTemplate) {}

	fn disconnected_user(&mut self, _: u64, template: &UserTemplate) {
		self.users.remove(&template.public_key);
	}
}

#[cfg(test)]
impl Default for Rooms {
	fn default() -> Self {
		Self {
			room_by_id: Default::default(),
			users: Rc::new(RefCell::new(Default::default())),
			changed_rooms: Default::default(),
			tracer: Rc::new(Tracer::new_with_allow_all()),
		}
	}
}
