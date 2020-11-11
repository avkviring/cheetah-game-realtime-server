use std::net::SocketAddr;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::network::udp::UDPServer;
use crate::rooms::Rooms;

pub struct Server {
	handler: JoinHandle<()>
}


impl Server {
	pub fn new(address: SocketAddr) -> Self {
		let handler = thread::spawn(move || {
			ServerThread::new(address).run();
		});
		Self {
			handler
		}
	}
	pub fn register_user() {}
}


struct ServerThread {
	udp_server: UDPServer,
	rooms: Rooms,
}

impl ServerThread {
	pub fn new(address: SocketAddr) -> Self {
		Self {
			udp_server: UDPServer::new(address).unwrap(),
			rooms: Default::default(),
		}
	}
	
	pub fn run(&mut self) {
		loop {
			let now = Instant::now();
			self.udp_server.cycle(&mut self.rooms);
			self.rooms.cycle(&now);
			thread::sleep(Duration::from_micros(500));
		}
	}
}
