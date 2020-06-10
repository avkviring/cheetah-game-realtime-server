use std::cell::RefCell;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use cheetah_relay::room::request::RoomRequest;
use cheetah_relay_client::{receive_commands_from_server, send_command_to_server};
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_client::client::ffi::CommandFFI;
use cheetah_relay_common::network::hash::HashValue;

use crate::integration::{add_wating_client_to_room, setup_client, setup_logger, setup_server};

#[test]
fn should_send_command_to_server() {
	setup_logger();
	let address = "127.0.0.1:6001";
	let client_hash = HashValue::from("client_hash");
	
	let (_, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	let client = setup_client(address, &room_hash, &client_hash);
	
	thread::sleep(Duration::from_secs(1));
	// upload object
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::Upload;
	ffi.object_id = 100;
	ffi.access_group = 0b100;
	send_command_to_server(client, &ffi);
	thread::sleep(Duration::from_secs(1));
	
	
	// check objects
	let rooms = &rooms.lock().unwrap();
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
	
	let (_, room_hash, rooms) = setup_server(address);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_a);
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_b);
	
	let client_a = setup_client(address, &room_hash, &client_hash_a);
	
	// upload object
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::Upload;
	ffi.object_id = 100;
	ffi.access_group = 0b100;
	send_command_to_server(client_a, &ffi);
	thread::sleep(Duration::from_secs(2));
	
	let client_b = setup_client(address, &room_hash, &client_hash_b);
	
	let mut collect_upload_command = Arc::new(AtomicBool::new(false));
	let mut move_arc = collect_upload_command.clone();
	// проверяем входящие команды на втором клиенте
	let collector = move |ffi: &CommandFFI| {
		if ffi.command_type_s2c == S2CCommandFFIType::Upload {
			(&*move_arc).store(true, Ordering::SeqCst);
		}
	};
	
	receive_commands_from_server(
		client_b,
		collector,
	);
	
	assert_eq!((&*collect_upload_command).load(Ordering::SeqCst), true);
}