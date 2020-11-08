use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use cheetah_relay_common::commands::hash::{RoomId, UserPublicKey};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandDescription, ApplicationCommands};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::{Room, RoomImpl};

#[derive(Default)]
pub struct Rooms {
	rooms: HashMap<RoomId, Rc<RefCell<RoomImpl>>>,
	user_to_room: HashMap<UserPublicKey, Rc<RefCell<RoomImpl>>>,
	changed_rooms: HashSet<RoomId>,
}


#[derive(Debug)]
pub struct OutFrame {
	pub user_public_key: UserPublicKey,
	pub frame: Frame,
}

pub enum RegisterUserError {
	RoomNotFound,
	AlreadyRegistered,
}

impl Rooms {
	pub fn create_room(&mut self, room_id: RoomId) {
		let room = RoomImpl::new(room_id.clone());
		self.rooms.insert(room_id, Rc::new(RefCell::new(room)));
	}
	
	pub fn register_user(&mut self, public_key: UserPublicKey, room_id: &RoomId, access_group: AccessGroups) -> Result<(), RegisterUserError> {
		match self.rooms.get(room_id) {
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
	
	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>) {
		self.changed_rooms.iter().for_each(|room_id| {
			let room = self.rooms.get(&room_id).unwrap().clone();
			let mut room = room.borrow_mut();
			room.collect_out_frames(out_frames);
		});
		self.changed_rooms.clear();
	}
	
	pub fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands) {
		on_user_room(self, user_public_key, |_, room| room.return_commands(user_public_key, commands));
	}
	
	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame) {
		on_user_room(self, user_public_key, |rooms, room|
			{
				room.on_frame_received(user_public_key, frame);
				rooms.changed_rooms.insert(room.id.clone());
			});
	}
}

fn on_user_room<F>(rooms: &mut Rooms, user_public_key: &UserPublicKey, mut action: F) where F: FnOnce(&mut Rooms, &mut RoomImpl) -> () {
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
