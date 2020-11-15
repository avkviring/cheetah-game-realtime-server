use std::thread;
use std::time::Duration;

use cheetah_relay_client::{do_receive_commands_from_server, do_send_command_to_server};
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, ChannelFFI, Command};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::helpers::Helper;

pub mod helpers;

#[test]
fn should_send_command_to_server() {
	let mut helper = Helper::new();
	let (mut server, addr, room) = helper.create_server_and_room();
	let user_a = helper.create_user_keys();
	let user_b = helper.create_user_keys();
	server.register_user(room, user_a.public, user_a.private, AccessGroups(0b100));
	server.register_user(room, user_b.public, user_b.private, AccessGroups(0b100));
	
	let client_a = helper.create_client(addr.to_string().as_str(), user_a.clone());
	let client_b = helper.create_client(addr.to_string().as_str(), user_b.clone());
	
	// upload object
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Create;
	ffi.object_id.set_from(&GameObjectId::new(100, ClientOwner::User(user_a.public)));
	ffi.channel = ChannelFFI::ReliableUnordered;
	ffi.access_group = 0b100;
	ffi.structures.count = 1;
	ffi.structures.fields[0] = 1;
	ffi.structures.sizes[0] = 2;
	ffi.structures.values[0] = 0x64;
	ffi.structures.values[1] = 0x65;
	
	
	do_send_command_to_server(client_a, &ffi, || assert!(false));
	ffi.command_type_c2s = C2SCommandFFIType::LoadRoom;
	//do_send_command_to_server(client_b, &ffi, || assert!(false));
	
	
	thread::sleep(Duration::from_secs(1));
	
	do_receive_commands_from_server(
		client_b,
		|command| {
			println!("command {:?}", command);
		},
		|| { assert!(false); },
	)
}
//
// #[test]
// fn should_receive_command_from_server() {
// 	setup_logger();
// 	let address = "127.0.0.1:6002";
// 	let client_hash_a = RoomId::from("client_hash_a");
// 	let client_hash_b = RoomId::from("client_hash_b");
//
// 	let (_server, room_hash, rooms) = setup_server(address);
// 	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_a);
// 	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_b);
//
// 	let client_a = setup_client(address, &room_hash, &client_hash_a);
//
// 	// upload object
// 	let mut ffi = Command::default();
// 	ffi.command_type_c2s = C2SCommandFFIType::Create;
// 	ffi.meta_timestamp = 123;
// 	ffi.object_id.set_from(&GameObjectId::new(100, ClientOwner::CurrentClient));
// 	ffi.access_group = 0b100;
// 	do_send_command_to_server(client_a, &ffi, || assert!(false));
// 	thread::sleep(Duration::from_secs(2));
//
// 	let client_b = setup_client(address, &room_hash, &client_hash_b);
//
//
// 	do_receive_commands_from_server(
// 		client_b,
// 		|command: &Command| {
// 			if command.command_type_s2c == S2CCommandFFIType::Create {
// 				assert_eq!(command.meta_timestamp, 123);
// 				assert!(command.meta_source_client != 0);
// 			} else {
// 				assert!(false);
// 			}
// 		},
// 		|| assert!(false),
// 	);
// }