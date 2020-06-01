use std::sync::mpsc::Receiver;

use crate::client::network::tcp::TCPClient;
use crate::client::request::ClientRequestType;

pub mod ffi;
pub mod command;
pub mod network;
pub mod request;

pub struct Client {
	receiver: Receiver<ClientRequestType>,
	tcp_client: TCPClient,
}


impl Client {
	pub fn new(
		server_address: String,
		receiver: Receiver<ClientRequestType>) -> Client {
		Client {
			receiver,
			tcp_client: TCPClient::new(),
		}
	}
	
	pub fn run(&mut self) {
		loop {
			self.tcp_client.cycle();
		}
	}
}
