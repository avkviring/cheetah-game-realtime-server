use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use cheetah_relay::room::request::RoomRequest;
use cheetah_relay_client::{receive_commands_from_server, send_command_to_server};
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, S2CCommandFFIType};
use cheetah_relay_client::client::ffi::Command;
use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::integration::{add_wating_client_to_room, setup_client, setup_logger, setup_server};

#[test]
fn should_send_command_to_server() {
	setup_logger();
	let address = "127.0.0.1:6001";
	let client_hash = HashValue::from("client_hash");
	
	let (_server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	let client = setup_client(address, &room_hash, &client_hash);
	
	thread::sleep(Duration::from_secs(1));
	// upload object
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Upload;
	ffi.object_id.set_from(&ClientGameObjectId::new(100, ClientOwner::CurrentClient));
	ffi.access_group = 0b100;
	ffi.structures.count = 1;
	ffi.structures.fields[0] = 1;
	ffi.structures.sizes[0] = 2;
	ffi.structures.values[0] = 0x64;
	ffi.structures.values[1] = 0x65;
	
	
	send_command_to_server(client, &ffi, || assert!(false));
	thread::sleep(Duration::from_secs(1));
	
	
	// check objects
	let rooms = &mut rooms.lock().unwrap();
	let (sender, receiver) = mpsc::channel();
	rooms.send_room_request(&room_hash, RoomRequest::GetObjects(sender)).ok().unwrap();
	let objects = receiver.recv().unwrap();
	assert_eq!(objects.is_empty(), false);
}

#[test]
fn should_receive_command_to_server() {
	setup_logger();
	let address = "127.0.0.1:6002";
	let client_hash_a = HashValue::from("client_hash_a");
	let client_hash_b = HashValue::from("client_hash_b");
	
	let (_server, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_a);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_b);
	
	let client_a = setup_client(address, &room_hash, &client_hash_a);
	
	// upload object
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Upload;
	ffi.object_id.set_from(&ClientGameObjectId::new(100, ClientOwner::CurrentClient));
	ffi.access_group = 0b100;
	send_command_to_server(client_a, &ffi, || assert!(false));
	thread::sleep(Duration::from_secs(2));
	
	let client_b = setup_client(address, &room_hash, &client_hash_b);
	
	
	receive_commands_from_server(
		client_b,
		|ffi: &Command| {
			if ffi.command_type_s2c == S2CCommandFFIType::Upload {
				assert!(true);
			} else {
				assert!(false);
			}
		},
		|| assert!(false),
	);
}