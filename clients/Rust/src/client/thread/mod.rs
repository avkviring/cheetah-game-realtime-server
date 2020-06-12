use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use cheetah_relay_common::network::hash::HashValue;

use crate::client::Client;
use crate::client::network::tcp::TCPClient;
use crate::client::request::{ClientRequestType, ExternalRequestProcessor};

pub struct ClientThread {
	client: Client,
	tcp_client: TCPClient,
	requests: ExternalRequestProcessor,
}

impl ClientThread {
	pub fn new(
		server_address: String,
		room_hash: HashValue,
		client_hash: HashValue,
		receiver: Receiver<ClientRequestType>) -> ClientThread {
		ClientThread {
			client: Client::new(room_hash, client_hash),
			tcp_client: TCPClient::new(server_address),
			requests: ExternalRequestProcessor::new(receiver),
		}
	}
	
	pub fn run(&mut self) {
		loop {
			let network_status = self.tcp_client.cycle(&mut self.client);
			self.client.network_status = network_status;
			self.requests.cycle(&mut self.client);
			thread::sleep(Duration::from_nanos(100));
		}
	}
}