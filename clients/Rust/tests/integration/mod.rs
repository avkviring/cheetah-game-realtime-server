use std::ffi::{CStr, CString};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use stderrlog::Timestamp;

use cheetah_relay::network::server::tcp::TCPServer;
use cheetah_relay::rooms::Rooms;
use cheetah_relay_client::create_client;

#[test]
fn test() {
	// let addr = "127.0.0.1:5001";
	// let (room, _) = setup(addr);
	// let addr_cstring = CString::from(addr);
	//
	// let clientA = create_client(
	// 	addr_cstring.as_ptr(),
	// 	|c| {}
	// );
	
	
	
}

fn setup(addr: &'static str) -> (Arc<Mutex<Rooms>>, JoinHandle<()>) {
	init_logger();
	let rooms = Arc::new(Mutex::new(Rooms::new()));
	let cloned_rooms = rooms.clone();
	let handle = thread::spawn(move || {
		let mut server = TCPServer::new(addr.to_string(), cloned_rooms);
		server.start();
	});
	(rooms, handle)
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.quiet(false)
		.show_level(true)
		.timestamp(Timestamp::Millisecond)
		.init();
}