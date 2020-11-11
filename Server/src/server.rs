use std::cmp::max;
use std::net::SocketAddr;
use std::ops::Sub;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cheetah_relay_common::room::{RoomId, UserPrivateKey, UserPublicKey};
use cheetah_relay_common::room::access::AccessGroups;

use crate::network::udp::UDPServer;
use crate::rooms::{RegisterRoomError, RegisterUserError, Rooms};

pub struct Server {
	handler: JoinHandle<()>,
	sender: Sender<Request>,
}

enum Request {
	RegisterRoom(RoomId, Sender<Result<(), RegisterRoomError>>),
	RegisterUser(RoomId, UserPublicKey, UserPrivateKey, AccessGroups, Sender<Result<(), RegisterUserError>>),
}


pub enum RegisterRoomRequestError {
	ChannelError(RecvTimeoutError),
	Error(RegisterRoomError),
}

pub enum RegisterUserRequestError {
	ChannelError(RecvTimeoutError),
	Error(RegisterUserError),
}

impl Server {
	pub fn new(address: SocketAddr, halt_signal: Arc<AtomicBool>) -> Self {
		let (sender, receiver) = std::sync::mpsc::channel();
		
		let handler = thread::spawn(move || { ServerThread::new(address, receiver, halt_signal).run(); });
		Self {
			handler,
			sender,
		}
	}
	
	
	pub fn register_room(&mut self, room_id: RoomId) -> Result<(), RegisterRoomRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(Request::RegisterRoom(room_id, sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => {
				match r {
					Ok(_) => {
						log::info!("create room {:?}", room_id);
						Result::Ok(())
					}
					Err(e) => {
						log::error!("fail create room {:?}", e);
						Result::Err(RegisterRoomRequestError::Error(e))
					}
				}
			}
			Err(e) => {
				log::error!("fail create room {:?}", e);
				Result::Err(RegisterRoomRequestError::ChannelError(e))
			}
		}
	}
	
	pub fn register_user(&mut self,
						 room_id: RoomId,
						 public_key: UserPublicKey,
						 private_key: UserPrivateKey,
						 access_groups: AccessGroups,
	) -> Result<(), RegisterUserRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(Request::RegisterUser(room_id, public_key, private_key, access_groups, sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => {
				match r {
					Ok(_) => {
						log::info!("create user {:?} in room {:?}", public_key, room_id);
						Result::Ok(())
					}
					Err(e) => {
						log::error!("fail create user {:?} in room {:?} with error {:?}", public_key, room_id, e);
						Result::Err(RegisterUserRequestError::Error(e))
					}
				}
			}
			Err(e) => {
				log::error!("fail create user {:?} in room {:?} with error {:?}", public_key, room_id, e);
				Result::Err(RegisterUserRequestError::ChannelError(e))
			}
		}
	}
	
	pub fn join(self) {
		self.handler.join().unwrap();
	}
}


struct ServerThread {
	udp_server: UDPServer,
	rooms: Rooms,
	receiver: Receiver<Request>,
	max_duration: u128,
	avg_duration: u128,
	halt_signal: Arc<AtomicBool>,
}

impl ServerThread {
	pub fn new(address: SocketAddr, receiver: Receiver<Request>, halt_signal: Arc<AtomicBool>) -> Self {
		Self {
			udp_server: UDPServer::new(address).unwrap(),
			rooms: Default::default(),
			receiver,
			max_duration: 0,
			avg_duration: 0,
			halt_signal,
		}
	}
	
	pub fn run(&mut self) {
		while self.halt_signal.load(Ordering::Relaxed) {
			let start_instant = Instant::now();
			self.udp_server.cycle(&mut self.rooms);
			self.rooms.cycle(&start_instant);
			self.do_request();
			self.statistics(start_instant);
		}
	}
	
	fn do_request(&mut self) {
		while let Ok(request) = self.receiver.try_recv() {
			match request {
				Request::RegisterRoom(room_id, sender) => {
					match sender.send(self.rooms.create_room(room_id)) {
						Ok(_) => {}
						Err(e) => {
							log::error!("[Request::RegisterRoom] error send response {:?}",e)
						}
					}
				}
				Request::RegisterUser(room_id, public_key, private_key, access_group, sender) => {
					self.udp_server.register_user(public_key, private_key);
					match sender.send(self.rooms.register_user(room_id, public_key, access_group)) {
						Ok(_) => {}
						Err(e) => {
							log::error!("[Request::RegisterUser] error send response {:?}",e)
						}
					}
				}
			}
		}
	}
	
	fn statistics(&mut self, start_instant: Instant) {
		let end_instant = Instant::now();
		let duration = end_instant.sub(start_instant).as_micros();
		if duration < 10 {
			thread::sleep(Duration::from_millis(1));
		}
		if self.avg_duration == 0 {
			self.avg_duration = duration;
		} else {
			self.avg_duration = (self.avg_duration + duration) / 2;
		}
		self.max_duration = max(self.max_duration, duration);
	}
}
