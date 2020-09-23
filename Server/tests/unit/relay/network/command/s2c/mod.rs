use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::SetFloat64CounterCommand;
use cheetah_relay_common::network::command::load::LoadGameObjectCommand;
use cheetah_relay_common::network::command::long_counter::SetLongCounterCommand;
use cheetah_relay_common::network::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::network::command::S2CCommandUnion;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use cheetah_relay::network::s2c::{AffectedClients, S2CCommandCollector};
use cheetah_relay::room::clients::{Client, Clients};
use cheetah_relay::room::listener::RoomListener;
use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};
use cheetah_relay::room::objects::object::GameObject;
use cheetah_relay::room::Room;

use crate::unit::relay::room::clients::client_stub_with_access_group;
use crate::unit::relay::room::room::room_stub;
use crate::unit::relay::room::setup_client;

#[test]
fn test_affects_client() {
	let groups = AccessGroups::from(0b111);
	let mut clients = Clients::default();
	let client = Rc::new(client_stub_with_access_group(0, 0b1));
	clients
		.clients
		.insert(0, client.clone());
	clients
		.clients
		.insert(1, Rc::new(client_stub_with_access_group(1, 0b100000)));
	clients
		.clients
		.insert(2, Rc::new(client_stub_with_access_group(2, 0b111)));
	
	let affected_client = AffectedClients::new_from_clients(&Option::Some(client), &clients, &groups);
	assert_eq!(affected_client.clients.contains(&0), false);
	assert_eq!(affected_client.clients.contains(&1), false);
	assert_eq!(affected_client.clients.contains(&2), true);
}

#[test]
fn should_s2c_collect_on_object_create() {
	let (mut room, collector) = setup();
	let _ = setup_client(&mut room, "HASH", 0b100);
	let (client, _, client_object_id, _) = setup_client_and_object(&mut room);
	
	let id = client.configuration.id;
	assert_command(
		collector,
		id,
		S2CCommandUnion::Load(LoadGameObjectCommand {
			object_id: client_object_id,
			template: 123,
			access_groups: AccessGroups::from(0b100),
			fields: Default::default(),
		}),
	);
}

#[test]
fn should_s2c_collect_on_client_connect() {
	let (mut room, commands) = setup();
	
	
	room.new_game_object(ServerGameObjectId::new(10, ServerOwner::Root), 123, AccessGroups::from(0b100), GameObjectFields::default()).unwrap();
	room.new_game_object(ServerGameObjectId::new(11, ServerOwner::Root), 123, AccessGroups::from(0b100), GameObjectFields::default()).unwrap();
	room.new_game_object(ServerGameObjectId::new(9, ServerOwner::Root), 123, AccessGroups::from(0b100), GameObjectFields::default()).unwrap();
	room.new_game_object(ServerGameObjectId::new(1, ServerOwner::Root), 123, AccessGroups::from(0b100), GameObjectFields::default()).unwrap();
	
	let client = setup_client(&mut room, "HASH", 0b100);
	let id = client.configuration.id;
	assert_command(
		commands.clone(),
		id,
		S2CCommandUnion::Load(LoadGameObjectCommand {
			object_id: ClientGameObjectId::new(10, ClientOwner::Root),
			template: 123,
			fields: Default::default(),
			access_groups: AccessGroups::from(0b100),
		}),
	);
	
	assert_command(
		commands,
		id,
		S2CCommandUnion::Load(LoadGameObjectCommand {
			object_id: ClientGameObjectId::new(11, ClientOwner::Root),
			template: 123,
			fields: Default::default(),
			access_groups: AccessGroups::from(0b100),
		}),
	);
}

#[test]
fn should_s2c_collect_on_client_disconnect() {
	let (mut room, commands) = setup();
	let client_a = setup_client(&mut room, "HASH_A", 0b100);
	room.new_game_object(
		ServerGameObjectId::new(1, ServerOwner::Client(client_a.configuration.id)),
		123,
		AccessGroups::from(0b100),
		GameObjectFields::default(),
	).unwrap();
	room.new_game_object(
		ServerGameObjectId::new(2, ServerOwner::Client(client_a.configuration.id)),
		123,
		AccessGroups::from(0b100),
		GameObjectFields::default(),
	).unwrap();
	room.new_game_object(
		ServerGameObjectId::new(3, ServerOwner::Client(client_a.configuration.id)),
		123,
		AccessGroups::from(0b100),
		GameObjectFields::default(),
	).unwrap();
	
	let client_b = setup_client(&mut room, "HASH_B", 0b100);
	clear_commands(commands.clone(), client_b.configuration.id);
	room.client_disconnect(&client_a);
	
	assert_command(
		commands.clone(),
		client_b.configuration.id,
		S2CCommandUnion::Unload(UnloadGameObjectCommand { object_id: ClientGameObjectId::new(1, ClientOwner::Client(client_a.configuration.id)) }),
	);
	
	assert_command(
		commands.clone(),
		client_b.configuration.id,
		S2CCommandUnion::Unload(UnloadGameObjectCommand {
			object_id: ClientGameObjectId::new(2, ClientOwner::Client(client_a.configuration.id)),
		}),
	);
	
	assert_command(
		commands,
		client_b.configuration.id,
		S2CCommandUnion::Unload(UnloadGameObjectCommand {
			object_id: ClientGameObjectId::new(3, ClientOwner::Client(client_a.configuration.id)),
		}),
	);
}

#[test]
fn should_s2c_collect_on_update_long_counter() {
	let (mut room, commands) = setup();
	let (client, _, client_object_id, object) = setup_client_and_object(&mut room);
	
	room.object_increment_long_counter(&mut object.borrow_mut(), 1, 155);
	clear_commands(commands.clone(), client.configuration.id);
	room.object_increment_long_counter(&mut object.borrow_mut(), 1, 55);
	
	assert_command(
		commands,
		client.configuration.id,
		S2CCommandUnion::SetLongCounter(SetLongCounterCommand {
			object_id: client_object_id,
			field_id: 1,
			value: 210,
		}),
	);
}


#[test]
fn should_s2c_collect_on_update_float_counter() {
	let (mut room, commands) = setup();
	let (client, _, client_object_id, object) = setup_client_and_object(&mut room);
	
	room.object_increment_float_counter(&mut object.borrow_mut(), 1, 155.0);
	clear_commands(commands.clone(), client.configuration.id);
	room.object_increment_float_counter(&mut object.borrow_mut(), 1, 55.0);
	
	assert_command(
		commands,
		client.configuration.id,
		S2CCommandUnion::SetFloatCounter(SetFloat64CounterCommand {
			object_id: client_object_id,
			field_id: 1,
			value: 210.0,
		}),
	);
}

#[test]
fn should_s2c_collect_on_fire_event() {
	let (mut room, commands) = setup();
	let (client, _, client_object_id, object) = setup_client_and_object(&mut room);
	clear_commands(commands.clone(), client.configuration.id);
	room.object_send_event(&mut object.borrow_mut(), 10, &vec![1, 2, 3, 4, 5]);
	
	assert_command(
		commands,
		client.configuration.id,
		S2CCommandUnion::Event(EventCommand {
			object_id: client_object_id,
			field_id: 10,
			event: vec![1, 2, 3, 4, 5],
		}),
	);
}

#[test]
fn should_s2c_collect_on_update_struct() {
	let (mut room, commands) = setup();
	let (client, _, client_object_id, object) = setup_client_and_object(&mut room);
	clear_commands(commands.clone(), client.configuration.id);
	room.object_update_struct(&mut object.borrow_mut(), 10, vec![1, 2, 3, 4, 5]);
	
	assert_command(
		commands,
		client.configuration.id,
		S2CCommandUnion::SetStruct(StructureCommand {
			object_id: client_object_id,
			field_id: 10,
			structure: vec![1, 2, 3, 4, 5],
		}),
	);
}

fn clear_commands(collector: Rc<RefCell<S2CCommandCollector>>, client_id: ClientId) {
	let mut collector = collector.borrow_mut();
	collector
		.commands_by_client
		.get_mut(&client_id)
		.map(|f| f.clear());
}

fn assert_command(
	collector: Rc<RefCell<S2CCommandCollector>>,
	client_id: ClientId,
	expected: S2CCommandUnion,
) {
	let mut collector = collector.borrow_mut();
	let commands = collector.commands_by_client.get_mut(&client_id).unwrap();
	assert_eq!(!commands.is_empty(), true);
	let actual = commands.pop_front().unwrap();
	assert_eq!(actual.command, expected)
}

fn setup() -> (Room, Rc<RefCell<S2CCommandCollector>>) {
	let mut room = room_stub();
	let collector = Rc::new(RefCell::new(S2CCommandCollector::default()));
	room.listener.add_listener(collector.clone());
	room.listener.set_current_meta_info(Rc::new(C2SMetaCommandInformation::new(0, 0)));
	(room, collector)
}

fn setup_client_and_object(mut room: &mut Room) -> (Rc<Client>, ServerGameObjectId, ClientGameObjectId, Rc<RefCell<GameObject>>) {
	let client = setup_client(&mut room, "HASH_A", 0b100);
	let server_object_id = ServerGameObjectId::new(1, ServerOwner::Client(client.configuration.id));
	let client_object_id = server_object_id.to_client_object_id(Option::Some(client.configuration.id));
	room
		.new_game_object(
			server_object_id.clone(),
			123,
			AccessGroups::from(0b100),
			GameObjectFields::default(),
		).unwrap();
	
	let object = room.objects.get(&server_object_id).unwrap();
	(client, server_object_id, client_object_id, object)
}