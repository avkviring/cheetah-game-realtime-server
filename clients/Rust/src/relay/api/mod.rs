use std::sync::Mutex;
use std::thread;

use crate::relay::client::Client;
use crate::relay::client::config::ClientConfig;
use crate::relay::client::controller::ClientController;

pub struct API {
	pub clients: Vec<Mutex<ClientController>>
}


impl API {
	pub fn new() -> API {
		API {
			clients: Default::default()
		}
	}
	
	pub fn create_client(
		&mut self,
		client_config: ClientConfig) -> u16 {
		let (sender, receiver) = std::sync::mpsc::channel();
		thread::spawn(move || {
			let mut client = Client::new(client_config, receiver);
			client.run()
		});
		self.clients.push(Mutex::new(ClientController::new(sender)));
		self.clients.len() as u16
	}
	
	pub fn destroy_client(
		&mut self,
		client_id: u16,
	) {
		unimplemented!()
	}
	
	
	pub fn collect_s2c_commands(
		&mut self,
		client_id: u16) {
		let client = self
			.clients
			.get(client_id as usize);
		if client.is_some() {
			let client = client.unwrap();
			let client = client.lock();
			let client = client.unwrap();
			//client.sender.send(ClientRequest::T1);
			unimplemented!()
		}
	}
}


