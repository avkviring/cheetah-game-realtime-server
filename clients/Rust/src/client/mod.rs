use std::collections::VecDeque;

use cheetah_relay_common::network::hash::HashValue;

use crate::client::command::{C2SCommandUnion, S2CCommandUnion};
use crate::client::request::{ClientRequestType, ExternalRequestProcessor};

pub mod ffi;
pub mod command;
pub mod network;
pub mod request;
pub mod thread;

#[derive(Debug)]
pub struct Client {
	pub room_hash: HashValue,
	pub client_hash: HashValue,
	pub network_status: NetworkStatus,
	pub scheduled_command_to_server: VecDeque<C2SCommandUnion>,
	pub commands_from_server: Vec<S2CCommandUnion>,
}


#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub enum NetworkStatus {
	///
	/// Сетевой канал в процессе установления соединения
	///
	Connecting,
	
	///
	/// Соединение с сервером установлено
	///
	OnLine,
	
	///
	/// Соединение разорвано
	///
	Disconnected,
}


impl Client {
	pub fn new(room_hash: HashValue,
			   client_hash: HashValue) -> Client {
		Client {
			room_hash,
			client_hash,
			network_status: NetworkStatus::Connecting,
			scheduled_command_to_server: Default::default(),
			commands_from_server: Default::default(),
		}
	}
	
	pub fn get_commands_from_server(&mut self) -> Vec<S2CCommandUnion> {
		self.commands_from_server.drain(..).collect()
	}
	
	pub fn schedule_command_to_server(&mut self, command: C2SCommandUnion) {
		self.scheduled_command_to_server.push_back(command);
	}
	
	pub fn close(&mut self) {}
}
