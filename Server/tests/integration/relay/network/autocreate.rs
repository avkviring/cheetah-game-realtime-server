use std::net::TcpStream;

use cheetah_relay::server::ServerBuilder;
use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::integration::relay::network::{create_client_and_send_hashes, get_clients, init_logger, send, setup};
use std::thread;
use std::time::Duration;

#[test]
fn should_connect_client_to_room() {
	let addr = "127.0.0.1:8000";
	init_logger();
	let server = ServerBuilder::new(addr.to_string()).enable_auto_create_room_and_client().build();
	thread::sleep(Duration::from_secs(1));
	let mut buffer = NioBuffer::new();
	let room_hash = HashValue::from("room");
	let client_hash = HashValue::from("client");
	create_client_and_send_hashes(&mut buffer, &room_hash, &client_hash);
	
	let mut stream = TcpStream::connect(addr).unwrap();
	send(&mut stream, &mut buffer);
	
	let clients = get_clients(server.rooms.clone(), &room_hash);
	assert_eq!(clients.iter().any(|p| p.hash == client_hash), true)
}