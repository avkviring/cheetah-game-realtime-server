use std::ffi::CString;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::Duration;

use stderrlog::Timestamp;

use cheetah_relay::room::request::{ClientInfo, RoomRequest};
use cheetah_relay::rooms::Rooms;
use cheetah_relay::server::{Server, ServerBuilder};
use cheetah_relay_client::create_client;
use cheetah_relay_common::commands::hash::RoomId;
use cheetah_relay_common::room::access::AccessGroups;

pub mod connect;
pub mod command;
pub mod disconnect;
pub mod benchmark;

fn get_server_room_clients(room_hash: &RoomId, rooms: Arc<Mutex<Rooms>>) -> Vec<ClientInfo> {
	let (sender, receiver) = mpsc::channel();
	let mut rooms = rooms.lock().unwrap();
	rooms.send_room_request(room_hash, RoomRequest::GetClients(sender)).ok().unwrap();
	receiver.recv().unwrap()
}


fn setup_client(address: &str, room_hash: &RoomId, client_hash: &RoomId) -> u16 {
	unsafe {
		let address = CString::new(address.to_string()).unwrap();
		let room_hash = CString::new(String::from(room_hash)).unwrap();
		let client_hash = CString::new(String::from(client_hash).as_str()).unwrap();
		let client = create_client(address.as_ptr(), room_hash.as_ptr(), client_hash.as_ptr());
		thread::sleep(Duration::from_secs(3));
		client
	}
}


fn setup_server(addr: &'static str) -> (Server, RoomId, Arc<Mutex<Rooms>>) {
	let room_hash = RoomId::from("room_hash");
	let server = ServerBuilder::new(addr.to_string()).build();
	let arc = server.rooms.clone();
	let rooms = arc;
	let rooms = &*rooms;
	let mut rooms = rooms.lock().unwrap();
	rooms.create_room(&room_hash);
	
	let rooms = server.rooms.clone();
	(server, room_hash, rooms)
}

fn add_wating_client_to_room(rooms: Arc<Mutex<Rooms>>, room_hash: &RoomId, client_hash: &RoomId) {
	let rooms = &*rooms;
	let mut rooms = rooms.lock().unwrap();
	rooms.send_room_request(
		&room_hash,
		RoomRequest::AddWaitingClient(client_hash.clone(), AccessGroups::from(0b111)),
	).ok().unwrap();
	
	thread::sleep(Duration::from_secs(1)); // ждем пока сервер запуститься
}

fn setup_logger() {
	stderrlog::new()
		.verbosity(0)
		.quiet(false)
		.show_level(true)
		.timestamp(Timestamp::Millisecond)
		.init();
}