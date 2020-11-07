use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cheetah_relay_common::commands::command::{C2SCommandWithMeta, S2CCommandWithMeta};
use cheetah_relay_common::commands::hash::RoomId;

pub mod ffi;
pub mod command;
pub mod network;
pub mod request;
pub mod thread;

#[derive(Debug)]
pub struct Client {
	pub room_hash: RoomId,
	pub client_hash: RoomId,
	pub network_status: Arc<Mutex<NetworkStatus>>,
	pub scheduled_command_to_server: VecDeque<C2SCommandWithMeta>,
	pub commands_from_server: Arc<Mutex<Vec<S2CCommandWithMeta>>>,
}


#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum NetworkStatus {
	None,
	///
	/// Сетевой канал в процессе установления соединения
	///
	Connecting,
	
	///
	/// Соединение с сервером установлено
	///
	Connected,
	
	///
	/// Соединение разорвано
	///
	Disconnected,
}

impl Client {
	pub fn new(room_hash: RoomId,
			   client_hash: RoomId,
			   commands_from_server: Arc<Mutex<Vec<S2CCommandWithMeta>>>,
			   network_status: Arc<Mutex<NetworkStatus>>) -> Client {
		Client {
			room_hash,
			client_hash,
			network_status,
			scheduled_command_to_server: Default::default(),
			commands_from_server,
		}
	}
	
	pub fn schedule_command_to_server(&mut self, command: C2SCommandWithMeta) {
		self.scheduled_command_to_server.push_back(command);
	}
}
