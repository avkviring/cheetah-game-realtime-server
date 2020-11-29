use std::cmp::max;
use std::net::UdpSocket;
use std::ops::{Add, Sub};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::{RoomId, UserPrivateKey, UserPublicKey};

use crate::network::udp::UDPServer;
use crate::room::object::GameObject;
use crate::rooms::{RegisterRoomError, RegisterUserError, Rooms};
use crate::server::dump::ServerDump;
use crate::server::Request::TimeOffset;

pub mod dump;
pub mod rest;

pub struct Server {
	handler: Option<JoinHandle<()>>,
	sender: Sender<Request>,
	halt_signal: Arc<AtomicBool>,
}

enum Request {
	RegisterRoom(RoomId, Sender<Result<(), RegisterRoomError>>),
	RegisterUser(RoomId, UserPublicKey, UserPrivateKey, AccessGroups, Sender<Result<(), RegisterUserError>>),
	///
	/// Смещение текущего времени для тестирования
	///
	TimeOffset(Duration),

	///
	/// Скопировать состояние сервера для отладки
	///
	Dump(Sender<ServerDump>),

	///
	/// Создать игровой объект в комнате
	///
	CreateObject(RoomId, GameObject),
}

pub enum RegisterRoomRequestError {
	ChannelError(RecvTimeoutError),
	Error(RegisterRoomError),
}

pub enum RegisterUserRequestError {
	ChannelError(RecvTimeoutError),
	Error(RegisterUserError),
}

impl Drop for Server {
	fn drop(&mut self) {
		self.halt_signal.store(true, Ordering::Relaxed);
	}
}

impl Server {
	pub fn new(socket: UdpSocket, auto_create_user: bool) -> Self {
		let (sender, receiver) = std::sync::mpsc::channel();
		let halt_signal = Arc::new(AtomicBool::new(false));
		let cloned_halt_signal = halt_signal.clone();
		let handler = thread::spawn(move || {
			ServerThread::new(socket, receiver, halt_signal, auto_create_user).run();
		});
		Self {
			handler: Option::Some(handler),
			sender,
			halt_signal: cloned_halt_signal,
		}
	}

	pub fn get_halt_signal(&self) -> Arc<AtomicBool> {
		self.halt_signal.clone()
	}

	pub fn register_room(&mut self, room_id: RoomId) -> Result<(), RegisterRoomRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(Request::RegisterRoom(room_id, sender)).unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => match r {
				Ok(_) => {
					log::info!("[server] create room({:?})", room_id);
					Result::Ok(())
				}
				Err(e) => {
					log::error!("[server] fail create room({:?})", e);
					Result::Err(RegisterRoomRequestError::Error(e))
				}
			},
			Err(e) => {
				log::error!("[server] fail create room({:?})", e);
				Result::Err(RegisterRoomRequestError::ChannelError(e))
			}
		}
	}

	pub fn register_user(
		&mut self,
		room_id: RoomId,
		public_key: UserPublicKey,
		private_key: UserPrivateKey,
		access_groups: AccessGroups,
	) -> Result<(), RegisterUserRequestError> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender
			.send(Request::RegisterUser(room_id, public_key, private_key, access_groups, sender))
			.unwrap();
		match receiver.recv_timeout(Duration::from_millis(100)) {
			Ok(r) => match r {
				Ok(_) => {
					log::info!("[server] create user({:?}) in room ({:?})", public_key, room_id);
					Result::Ok(())
				}
				Err(e) => {
					log::error!("[server] fail create user ({:?}) in room ({:?}) with error {:?}", public_key, room_id, e);
					Result::Err(RegisterUserRequestError::Error(e))
				}
			},
			Err(e) => {
				log::error!("[server] fail create user ({:?}) in room ({:?}) with error {:?}", public_key, room_id, e);
				Result::Err(RegisterUserRequestError::ChannelError(e))
			}
		}
	}

	pub fn set_time_offset(&self, duration: Duration) {
		self.sender.send(TimeOffset(duration)).unwrap();
	}

	pub fn join(&mut self) {
		self.handler.take().unwrap().join().unwrap();
	}

	pub fn dump(&self) -> Result<ServerDump, ()> {
		let (sender, receiver) = std::sync::mpsc::channel();
		self.sender.send(Request::Dump(sender)).unwrap();
		receiver.recv().map_err(|_| ())
	}

	pub fn create_object(&self, room: RoomId, object: GameObject) -> Result<(), ()> {
		self.sender.send(Request::CreateObject(room, object)).map_err(|_| ())
	}
}

struct ServerThread {
	udp_server: UDPServer,
	rooms: Rooms,
	receiver: Receiver<Request>,
	max_duration: u128,
	avg_duration: u128,
	halt_signal: Arc<AtomicBool>,
	time_offset: Option<Duration>,
}

impl ServerThread {
	pub fn new(socket: UdpSocket, receiver: Receiver<Request>, halt_signal: Arc<AtomicBool>, auto_create_user: bool) -> Self {
		Self {
			udp_server: UDPServer::new(socket, auto_create_user).unwrap(),
			rooms: Rooms::new(auto_create_user),
			receiver,
			max_duration: 0,
			avg_duration: 0,
			halt_signal,
			time_offset: None,
		}
	}

	pub fn run(&mut self) {
		while !self.halt_signal.load(Ordering::Relaxed) {
			let mut now = Instant::now();
			if let Some(time_offset) = self.time_offset {
				now = now.add(time_offset);
			}
			self.udp_server.cycle(&mut self.rooms, &now);
			self.rooms.cycle(&now);
			self.do_request();
			self.statistics(now);
		}
	}

	fn do_request(&mut self) {
		while let Ok(request) = self.receiver.try_recv() {
			match request {
				Request::RegisterRoom(room_id, sender) => match sender.send(self.rooms.create_room(room_id)) {
					Ok(_) => {}
					Err(e) => {
						log::error!("[Request::RegisterRoom] error send response {:?}", e)
					}
				},
				Request::RegisterUser(room_id, public_key, private_key, access_group, sender) => {
					self.udp_server.register_user(public_key, private_key);
					match sender.send(self.rooms.register_user(room_id, public_key, access_group)) {
						Ok(_) => {}
						Err(e) => {
							log::error!("[Request::RegisterUser] error send response {:?}", e)
						}
					}
				}
				TimeOffset(time_offset) => {
					self.time_offset = Option::Some(time_offset);
				}
				Request::Dump(sender) => {
					sender.send(ServerDump::from(&*self)).unwrap();
				}
				Request::CreateObject(room_id, object) => {
					let room = self.rooms.room_by_id.get_mut(&room_id).unwrap();
					room.borrow_mut().objects.insert(object.id.clone(), object);
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
