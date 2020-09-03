use std::thread;
use std::time::Duration;

use cheetah_relay::room::request::{RoomRequest, RoomRequests};
use cheetah_relay_client::{destroy_client, get_connection_status};
use cheetah_relay_client::client::NetworkStatus;
use cheetah_relay_common::network::hash::HashValue;

use crate::integration::{add_wating_client_to_room, setup_client, setup_logger, setup_server};

#[test]
fn should_disconnect_to_server_when_server_closed() {
	setup_logger();
	let address = "127.0.0.1:7002";
	let client_hash = HashValue::from("client_hash");
	
	let (mut server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	
	let client = setup_client(address, &room_hash, &client_hash);
	get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Connected); },
		|| { assert!(false) },
	);
	drop(server);
	thread::sleep(Duration::from_secs(1));
	get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Disconnected); },
		|| { assert!(false) },
	);
}

#[test]
fn should_disconnect_client() {
	setup_logger();
	let address = "127.0.0.1:7003";
	let client_hash = HashValue::from("client_hash");
	
	let (mut server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	
	let client = setup_client(address, &room_hash, &client_hash);
	get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Connected); },
		|| { assert!(false) },
	);
	destroy_client(client);
	thread::sleep(Duration::from_secs(1));
	
	let mut rooms = server.rooms.lock().unwrap();
	let (sender, receiver) = std::sync::mpsc::channel();
	rooms.send_room_request(&room_hash, RoomRequest::GetClients(sender));
	let result = receiver.recv().unwrap();
	assert_eq!(true, result.is_empty())
}