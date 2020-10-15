use cheetah_relay_client::client::NetworkStatus;
use cheetah_relay_client::do_get_connection_status;
use cheetah_relay_common::commands::hash::HashValue;

use crate::integration::{add_wating_client_to_room, get_server_room_clients, setup_client, setup_logger, setup_server};

#[test]
fn should_fail_connect_to_server_when_server_not_running() {
	let address = "127.0.0.1:5001";
	let room_hash = HashValue::from("room_hash");
	let client_hash = HashValue::from("client_hash");
	let client = setup_client(address, &room_hash, &client_hash);
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Disconnected); },
		|| { assert!(false) },
	);
}

#[test]
fn should_connect_to_server() {
	setup_logger();
	let address = "127.0.0.1:5002";
	let client_hash = HashValue::from("client_hash");
	let (_server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms, &room_hash, &client_hash);
	let client = setup_client(address, &room_hash, &client_hash);
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Connected); },
		|| { assert!(false) },
	);
}

#[test]
fn should_connect_to_room_server() {
	let address = "127.0.0.1:5003";
	let client_hash = HashValue::from("client_hash");
	
	let (_server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	
	let client = setup_client(address, &room_hash, &client_hash);
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, NetworkStatus::Connected); },
		|| { assert!(false) },
	);
	
	let clients = get_server_room_clients(&room_hash, rooms.clone());
	assert_eq!(clients.iter().any(|c| c.hash == client_hash), true);
}
