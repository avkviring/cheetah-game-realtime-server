// use cheetah_relay_common::room::access::AccessGroups;
// use cheetah_relay_common::room::owner::ClientOwner;
// use cheetah_relay_common::room::object::GameObjectId;
//
// use crate::helpers::Helper;
// pub mod helpers;
//
// #[test]
// fn should_send_command_to_server() {
// 	let mut helper = Helper::new();
// 	let (mut server, addr, room) = helper.create_server_and_room();
// 	let user_a = helper.create_user_keys();
// 	let user_b = helper.create_user_keys();
// 	server.register_user(room, user_a.public, user_a.private, AccessGroups(0b100));
// 	server.register_user(room, user_b.public, user_b.private, AccessGroups(0b100));
//
// 	let client_a = helper.create_client(addr.to_string().as_str(), user_a.clone());
// 	let client_b = helper.create_client(addr.to_string().as_str(), user_b.clone());
//
// 	// upload object
// 	let mut ffi = Command::default();
// 	ffi.command_type_c2s = C2SCommandFFIType::Create;
// 	ffi.object_id.set_from(&GameObjectId::new(100, ClientOwner::User(user_a.public)));
// 	ffi.channel = ChannelFFI::ReliableUnordered;
// 	ffi.access_group = 0b100;
// 	ffi.structures.count = 1;
// 	ffi.structures.fields[0] = 1;
// 	ffi.structures.sizes[0] = 2;
// 	ffi.structures.values[0] = 0x64;
// 	ffi.structures.values[1] = 0x65;
//
//
// 	do_send_command_to_server(client_a, &ffi, || assert!(false));
// 	ffi.command_type_c2s = C2SCommandFFIType::LoadRoom;
// 	//do_send_command_to_server(client_b, &ffi, || assert!(false));
//
//
// 	thread::sleep(Duration::from_secs(1));
//
// 	do_receive_commands_from_server(
// 		client_b,
// 		|command| {
// 			println!("command {:?}", command);
// 		},
// 		|| { assert!(false); },
// 	)
// }
