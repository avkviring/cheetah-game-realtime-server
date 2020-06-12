use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{Builder, JoinHandle};

use crate::network::server::tcp::{TCPAcceptor, TCPAcceptorRequest};
use crate::room::request::RoomRequest;
use crate::rooms::Rooms;

pub struct Server {
	pub rooms: Arc<Mutex<Rooms>>,
	pub tcp_acceptor_handler: JoinHandle<()>,
	pub sender: Sender<TCPAcceptorRequest>,
}

impl Server {
	pub fn new(listen_address: String) -> Self {
		let (sender, receiver) = std::sync::mpsc::channel();
		
		let rooms = Arc::new(Mutex::new(Rooms::default()));
		let cloned_rooms = rooms.clone();
		let tcp_acceptor_handler = Builder::new()
			.name("tcp acceptor".to_string())
			.spawn(move || {
				let mut server = TCPAcceptor::new(listen_address, cloned_rooms, receiver);
				server.start();
			}).unwrap();
		
		Server {
			rooms,
			tcp_acceptor_handler,
			sender,
		}
	}
	
	pub fn close(mut self) {
		println!("server: close");
		self.sender.send(TCPAcceptorRequest::Close).unwrap();
		self.tcp_acceptor_handler.join();
		let rooms = &mut self.rooms.lock().unwrap();
		rooms.close_all_rooms();
	}
}




