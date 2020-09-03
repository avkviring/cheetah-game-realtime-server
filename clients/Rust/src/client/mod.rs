use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use cheetah_relay_common::network::hash::HashValue;

use crate::client::command::{C2SCommandUnion, S2CCommandUnion};

pub mod ffi;
pub mod command;
pub mod network;
pub mod request;
pub mod thread;

#[derive(Debug)]
pub struct Client {
	pub room_hash: HashValue,
	pub client_hash: HashValue,
	pub network_status: Arc<Mutex<NetworkStatus>>,
	pub scheduled_command_to_server: VecDeque<C2SCommandUnion>,
	pub commands_from_server: Arc<Mutex<Vec<S2CCommandUnion>>>,
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
	pub fn new(room_hash: HashValue,
			   client_hash: HashValue,
			   commands_from_server: Arc<Mutex<Vec<S2CCommandUnion>>>,
			   network_status: Arc<Mutex<NetworkStatus>>) -> Client {
		Client {
			room_hash,
			client_hash,
			network_status,
			scheduled_command_to_server: Default::default(),
			commands_from_server,
		}
	}
	
	pub fn schedule_command_to_server(&mut self, command: C2SCommandUnion) {
		self.scheduled_command_to_server.push_back(command);
	}
}
