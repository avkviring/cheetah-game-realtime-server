use cheetah_relay_common::commands::hash::HashValue;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;
use easybench::bench;

use cheetah_relay_client::{do_receive_commands_from_server, do_send_command_to_server};
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Command, S2CCommandFFIType};

use crate::integration::{add_wating_client_to_room, setup_client, setup_logger, setup_server};

#[test]
fn benchmark_send_command() {
	setup_logger();
	let address = "127.0.0.1:9001";
	let (_server, room_hash, rooms) = setup_server(address);
	
	let client_hash = HashValue::from("client");
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash);
	let client = setup_client(address, &room_hash, &client_hash);
	
	let object_id = create_object_on_server(client);
	
	let result = bench(|| {
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::SetFloatCounter;
		ffi.object_id.set_from(&object_id);
		
		let count = 100;
		for i in 0..count {
			ffi.field_id = i;
			do_send_command_to_server(client, &ffi, || assert!(false));
		}
	});
	
	println!("{}", result);
	assert!(result.ns_per_iter < sec_to_nano(0.000_300_000));
}


///
/// Время отправки и приема простых команд
///
#[test]
fn benchmark_send_and_receive_commands() {
	setup_logger();
	let address = "127.0.0.1:9002";
	let (_server, room_hash, rooms) = setup_server(address);
	
	let client_hash_a = HashValue::from("client_hash_a");
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_a);
	let client_a = setup_client(address, &room_hash, &client_hash_a);
	
	let client_hash_b = HashValue::from("client_hash_b");
	add_wating_client_to_room(rooms.clone(), &room_hash, &client_hash_b);
	let client_b = setup_client(address, &room_hash, &client_hash_b);
	
	
	let object_id = create_object_on_server(client_a);
	
	
	let result = bench(|| {
		
		// отправляем команды
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::SetFloatCounter;
		ffi.object_id.set_from(&object_id);
		
		let count = 100;
		for i in 0..count {
			ffi.field_id = i;
			do_send_command_to_server(client_a, &ffi, || assert!(false));
		}
		let mut recv_count = 0;
		while recv_count < count {
			do_receive_commands_from_server(
				client_b,
				|ffi: &Command| {
					match ffi.command_type_s2c {
						S2CCommandFFIType::Create => {}
						S2CCommandFFIType::SetFloatCounter => {
							recv_count = recv_count + 1;
						}
						_ => {
							assert!(false);
						}
					}
				},
				|| assert!(false),
			);
		};
	});
	
	println!("{}", result);
	assert!(result.ns_per_iter < sec_to_nano(0.07));
}


fn create_object_on_server(client: u16) -> ClientGameObjectId {
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Create;
	let object_id = ClientGameObjectId::new(100, ClientOwner::CurrentClient);
	ffi.object_id.set_from(&object_id);
	ffi.access_group = 0b100;
	do_send_command_to_server(client, &ffi, || assert!(false));
	object_id
}

fn sec_to_nano(sec: f64) -> f64 {
	return sec * 1_000_000_000.0;
}