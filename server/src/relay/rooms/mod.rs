
use std::collections::HashMap;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Sender, SendError};
use std::thread;
use std::thread::JoinHandle;

use crate::relay::room::request::RoomRequest;
use crate::relay::room::cycle::RoomCycle;
use crate::relay::network::types::hash::HashValue;

/// комнаты
pub struct Rooms {
	registry: Arc<Mutex<HashMap<HashValue, RoomThreadController>>>
}

/// каналы для управления Room
pub struct RoomThreadController {
	pub sender: Sender<RoomRequest>,
	pub handle: JoinHandle<()>,
}

pub enum SendRoomRequestError {
	RoomNotFound,
	SendError(SendError<RoomRequest>),
}


impl Rooms {
	pub fn new() -> Rooms {
		Rooms {
			registry: Default::default()
		}
	}
	
	pub fn create_room(&mut self, room_hash: &HashValue) {
		let (sender, receiver) = mpsc::channel();
		
		let cloned_room_hash = room_hash.clone();
		let handle = thread::spawn(|| {
			let mut room_cycle = RoomCycle::new(cloned_room_hash, receiver);
			room_cycle.start();
		});
		
		
		let registry = &*self.registry.clone();
		let mut registry = registry.lock().unwrap();
		registry.insert(room_hash.clone(), RoomThreadController {
			sender,
			handle,
		});
	}
	
	pub fn send_room_request(&self, room_hash: &HashValue, request: RoomRequest) -> Result<(), SendRoomRequestError> {
		let registry = &*self.registry.clone();
		let registry = registry.lock().unwrap();
		let room = registry.get(room_hash);
		if room.is_some() {
			match room.unwrap().sender.send(request) {
				Ok(_) => {
					Result::Ok(())
				}
				Err(error) => {
					Result::Err(SendRoomRequestError::SendError(error))
				}
			}
		} else {
			Result::Err(SendRoomRequestError::RoomNotFound)
		}
	}
}