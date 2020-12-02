use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::template::{RoomTemplate, UserTemplate};
use crate::room::{Room, RoomId};

#[derive(Default, Clone)]
pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Rc<RefCell<Room>>, FnvBuildHasher>,
	pub user_to_room: HashMap<UserPublicKey, Rc<RefCell<Room>>, FnvBuildHasher>,
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
			let rc = Rc::new(RefCell::new(room));
			self.room_by_id.insert(room_id, rc.clone());
			config.users.iter().for_each(|config| {
				self.user_to_room.insert(config.public_key, rc.clone());
			});
			Result::Ok(())
		}
	}

	pub fn register_user(&mut self, room_id: RoomId, template: UserTemplate) -> Result<(), RegisterUserError> {
		match self.room_by_id.get(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				let public_key = template.public_key;
				if !(self.user_to_room.contains_key(&public_key)) {
					let room = room.clone();
					room.borrow_mut().register_user(template);
					self.user_to_room.insert(public_key, room);
					Result::Ok(())
				} else {
					Result::Err(RegisterUserError::AlreadyRegistered)
				}
			}
		}
	}

	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		self.changed_rooms.iter().for_each(|room_id| {
			let room = self.room_by_id.get(room_id).unwrap();
			let mut room = room.borrow_mut();
			room.collect_out_frame(out_frames, now);
		});
		self.changed_rooms.clear();
	}

	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		on_user_room(self, user_public_key, |rooms, room| {
			room.process_in_frame(user_public_key, frame, now);
			rooms.changed_rooms.insert(room.id);
		});
	}

	pub fn cycle(&mut self, now: &Instant) {
		self.room_by_id.values().for_each(|r| r.borrow_mut().cycle(now));
	}
}

fn on_user_room<F>(rooms: &mut Rooms, user_public_key: &UserPublicKey, action: F)
where
	F: FnOnce(&mut Rooms, &mut Room),
{
	match rooms.user_to_room.get(user_public_key) {
		None => {
			log::error!("[rooms] user({:?} not found ", user_public_key);
		}
		Some(room) => {
			let room = room.clone();
			let mut room = room.borrow_mut();
			action(rooms, &mut room);
		}
	}
}
