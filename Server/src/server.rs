use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread::{Builder, JoinHandle};
use crate::network::rooms::Rooms;

//use crate::rooms::Rooms;

pub struct Server {
	pub rooms: Arc<Mutex<Rooms>>,
	pub tcp_acceptor_handler: Option<JoinHandle<()>>,
	//pub sender: Sender<TCPAcceptorRequest>,
}

pub struct ServerBuilder {
	auto_create_rooms_and_clients: bool,
	listen_address: String,
}

impl ServerBuilder {
	pub fn new(listen_address: String) -> Self {
		ServerBuilder {
			auto_create_rooms_and_clients: false,
			listen_address,
		}
	}
	
	pub fn enable_auto_create_room_and_client(mut self) -> Self {
		self.auto_create_rooms_and_clients = true;
		self
	}
	
	
	pub fn build(self) -> Server {
		Server::new(self.listen_address, self.auto_create_rooms_and_clients)
	}
}


impl Server {
	pub fn new(listen_address: String, auto_create_rooms_and_clients: bool) -> Self {
		// //let (sender, receiver) = std::sync::mpsc::channel();
		//
		// let rooms = Arc::new(Mutex::new(Rooms::new(auto_create_rooms_and_clients)));
		// let cloned_rooms = rooms.clone();
		// let tcp_acceptor_handler = Builder::new()
		// 	.name("tcp acceptor".to_string())
		// 	.spawn(move || {
		// 		// let mut server = TCPAcceptor::new(listen_address, cloned_rooms, receiver);
		// 		// server.start();
		// 	}).unwrap();
		//
		// Server {
		// 	rooms,
		// 	tcp_acceptor_handler: Option::Some(tcp_acceptor_handler),
		// 	//sender,
		// }
		panic!();
	}
}


impl Drop for Server {
	fn drop(&mut self) {
		//self.sender.send(TCPAcceptorRequest::Close).unwrap();
		let rooms = &mut self.rooms.lock().unwrap();
		rooms.close_all_rooms();
		self.tcp_acceptor_handler.take().unwrap().join().unwrap();
	}
}

