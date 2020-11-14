use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use fnv::FnvBuildHasher;

use cheetah_relay_common::protocol::frame::applications::ApplicationCommands;
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::room::{RoomId, UserPublicKey};
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::Room;

#[derive(Default)]
pub struct Rooms {
	rooms: HashMap<RoomId, Rc<RefCell<Room>>, FnvBuildHasher>,
	user_to_room: HashMap<UserPublicKey, Rc<RefCell<Room>>, FnvBuildHasher>,
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
	AlreadyRegistered
}

impl Rooms {
	pub fn create_room(&mut self, room_id: RoomId) -> Result<(), RegisterRoomError> {
		if self.rooms.contains_key(&room_id) {
			Result::Err(RegisterRoomError::AlreadyRegistered)
		} else {
			let room = Room::new(room_id);
			self.rooms.insert(room_id, Rc::new(RefCell::new(room)));
			Result::Ok(())
		}
	}
	
	pub fn register_user(&mut self, room_id: RoomId, public_key: UserPublicKey, access_group: AccessGroups) -> Result<(), RegisterUserError> {
		match self.rooms.get(&room_id) {
			None => {
				Result::Err(RegisterUserError::RoomNotFound)
			}
			Some(room) => if !(self.user_to_room.contains_key(&public_key)) {
				let room = room.clone();
				room.borrow_mut().register_user(public_key, access_group);
				self.user_to_room.insert(public_key, room);
				Result::Ok(())
			} else {
				Result::Err(RegisterUserError::AlreadyRegistered)
			},
		}
	}
	
	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		self.changed_rooms.iter().for_each(|room_id| {
			let room = self.rooms.get(&room_id).unwrap().clone();
			let mut room = room.borrow_mut();
			room.collect_out_frame(out_frames, now);
		});
		self.changed_rooms.clear();
	}
	
	pub fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands) {
		on_user_room(self, user_public_key, |_, room| room.send_to_user_first(user_public_key, commands));
	}
	
	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		on_user_room(self, user_public_key, |rooms, room|
			{
				room.process_in_frame(user_public_key, frame, now);
				rooms.changed_rooms.insert(room.id.clone());
			});
	}
	
	pub fn cycle(&mut self, now: &Instant) {
		self.rooms.values().for_each(|r| r.clone().borrow_mut().cycle(now));
	}
}

fn on_user_room<F>(rooms: &mut Rooms, user_public_key: &UserPublicKey, action: F) where F: FnOnce(&mut Rooms, &mut Room) {
	let room = rooms.user_to_room.get(user_public_key);
	match room {
		None => {
			log::error!("room for user not found for {:?}", user_public_key);
		}
		Some(room) => {
			let room = room.clone();
			let mut room = room.borrow_mut();
			action(rooms, &mut room);
		}
	}
}
