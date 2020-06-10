use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::network::server::tcp::TCPServer;
use crate::rooms::Rooms;

pub struct Server {
	pub rooms: Arc<Mutex<Rooms>>,
	pub handle: JoinHandle<()>,
}

impl Server {
	pub fn new(listen_address: String) -> Self {
		let rooms = Arc::new(Mutex::new(Rooms::new()));
		let cloned_rooms = rooms.clone();
		let handle = thread::spawn(move || {
			let mut server = TCPServer::new(listen_address, cloned_rooms);
			server.start();
		});
		Server {
			rooms,
			handle,
		}
	}
}




