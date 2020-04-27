use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;
use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};
use crate::relay::network::command::s2c::delete_game_object::DeleteGameObjectS2CCommand;
use crate::relay::network::command::s2c::event::EventS2CCommand;
use crate::relay::network::command::s2c::S2CCommandCollector;
use crate::relay::network::command::s2c::update_float_counter::UpdateFloatCounterS2CCommand;
use crate::relay::network::command::s2c::update_long_counter::UpdateLongCounterS2CCommand;
use crate::relay::network::command::s2c::upload_object::UploadGameObjectS2CCommand;
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::{GameObject, GameObjectTemplate};
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::Room;
use crate::test::relay::room::setup_client;
use crate::relay::network::command::s2c::update_struct::UpdateStructS2CCommand;

#[test]
fn test_affects_client() {
	let groups = AccessGroups::from(0b111);
	let mut clients = Clients::default();
	clients.clients.insert(0, Rc::new(Client::stub_with_access_group(0, 0b1)));
	clients.clients.insert(1, Rc::new(Client::stub_with_access_group(1, 0b100000)));
	clients.clients.insert(2, Rc::new(Client::stub_with_access_group(2, 0b111)));
	
	let affected_client = AffectedClients::new_from_clients(&clients, &groups);
	assert_eq!(affected_client.clients.contains(&0), true);
	assert_eq!(affected_client.clients.contains(&1), false);
	assert_eq!(affected_client.clients.contains(&2), true);
}

#[test]
fn should_s2c_collect_on_object_create() {
	let (mut room, commands) = setup();
	let client = setup_client(&mut room, "HASH", 0b100);
	room.create_root_game_object(10, &GameObjectTemplate::stub_with_group(0b100));
	
	assert_command(commands, &UploadGameObjectS2CCommand {
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		cloned_object: GameObject {
			id: 10,
			owner: Owner::new_root_owner(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups: AccessGroups::from(0b100),
		},
	});
}

#[test]
fn should_s2c_collect_on_client_connect() {
	let (mut room, commands) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub_with_group(0b100));
	room.create_root_game_object(11, &GameObjectTemplate::stub_with_group(0b100));
	room.create_root_game_object(9, &GameObjectTemplate::stub_with_group(0b100));
	room.create_root_game_object(1, &GameObjectTemplate::stub_with_group(0b100));
	clear_commands(commands.clone());
	
	let client = setup_client(&mut room, "HASH", 0b100);
	
	assert_command(commands.clone(), &UploadGameObjectS2CCommand {
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		cloned_object: GameObject {
			id: 10,
			owner: Owner::new_root_owner(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups: AccessGroups::from(0b100),
		},
	});
	
	assert_command(commands.clone(), &UploadGameObjectS2CCommand {
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		cloned_object: GameObject {
			id: 11,
			owner: Owner::new_root_owner(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			groups: AccessGroups::from(0b100),
		},
	});
}

#[test]
fn should_s2c_collect_on_client_disconnect() {
	let (mut room, commands) = setup();
	
	let clientA = setup_client(&mut room, "HASH_A", 0b100);
	room.create_client_game_object(&clientA, 1, &GameObjectTemplate::stub_with_group(0b100));
	room.create_client_game_object(&clientA, 2, &GameObjectTemplate::stub_with_group(0b100));
	room.create_client_game_object(&clientA, 3, &GameObjectTemplate::stub_with_group(0b100));
	let clientB = setup_client(&mut room, "HASH_B", 0b100);
	clear_commands(commands.clone());
	room.client_disconnect(&clientA);
	assert_command(commands.clone(), &DeleteGameObjectS2CCommand {
		global_object_id: GameObject::to_global_object_id(&clientA, 1),
		affected_clients: AffectedClients { clients: vec![clientB.configuration.id] },
	});
	assert_command(commands.clone(), &DeleteGameObjectS2CCommand {
		global_object_id: GameObject::to_global_object_id(&clientA, 2),
		affected_clients: AffectedClients { clients: vec![clientB.configuration.id] },
	});
	assert_command(commands.clone(), &DeleteGameObjectS2CCommand {
		global_object_id: GameObject::to_global_object_id(&clientA, 3),
		affected_clients: AffectedClients { clients: vec![clientB.configuration.id] },
	});
}

#[test]
fn should_s2c_collect_on_update_long_counter() {
	let (mut room, commands) = setup();
	let client = setup_client(&mut room, "HASH_A", 0b100);
	let id = room.create_client_game_object(&client, 1, &GameObjectTemplate::stub_with_group(0b100)).ok().unwrap();
	let object = room.objects.get(id).unwrap();
	let object = object.clone();
	let object = &*object;
	room.object_increment_long_counter(&mut object.borrow_mut(), 1, 155);
	clear_commands(commands.clone());
	room.object_increment_long_counter(&mut object.borrow_mut(), 1, 55);
	
	assert_command(commands.clone(), &UpdateLongCounterS2CCommand {
		global_object_id: id,
		field_id: 1,
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		value: 210,
	});
}

#[test]
fn should_s2c_collect_on_update_float_counter() {
	let (mut room, commands) = setup();
	let client = setup_client(&mut room, "HASH_A", 0b100);
	let id = room.create_client_game_object(&client, 1, &GameObjectTemplate::stub_with_group(0b100)).ok().unwrap();
	let object = room.objects.get(id).unwrap();
	let object = object.clone();
	let object = &*object;
	room.object_increment_float_counter(&mut object.borrow_mut(), 1, 155.0);
	clear_commands(commands.clone());
	room.object_increment_float_counter(&mut object.borrow_mut(), 1, 55.0);
	
	assert_command(commands.clone(), &UpdateFloatCounterS2CCommand {
		global_object_id: id,
		field_id: 1,
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		value: 210.0,
	});
}

#[test]
fn should_s2c_collect_on_fire_event() {
	let (mut room, commands) = setup();
	let client = setup_client(&mut room, "HASH_A", 0b100);
	let id = room.create_client_game_object(&client, 1, &GameObjectTemplate::stub_with_group(0b100)).ok().unwrap();
	let object = room.objects.get(id).unwrap();
	let object = object.clone();
	let object = &*object;
	clear_commands(commands.clone());
	room.object_send_event(&mut object.borrow_mut(), 10, &vec![1, 2, 3, 4, 5]);
	
	assert_command(commands.clone(), &EventS2CCommand {
		global_object_id: id,
		field_id: 10,
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		event: vec![1, 2, 3, 4, 5],
	});
}

#[test]
fn should_s2c_collect_on_update_struct() {
	let (mut room, commands) = setup();
	let client = setup_client(&mut room, "HASH_A", 0b100);
	let id = room.create_client_game_object(&client, 1, &GameObjectTemplate::stub_with_group(0b100)).ok().unwrap();
	let object = room.objects.get(id).unwrap();
	let object = object.clone();
	let object = &*object;
	clear_commands(commands.clone());
	room.object_update_struct(&mut object.borrow_mut(), 10, &vec![1, 2, 3, 4, 5]);
	
	assert_command(commands.clone(), &UpdateStructS2CCommand {
		global_object_id: id,
		field_id: 10,
		affected_clients: AffectedClients { clients: vec![client.configuration.id] },
		struct_data: vec![1, 2, 3, 4, 5],
	});
}


fn clear_commands(commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>) {
	let commands = commands.clone();
	let mut commands = (&*commands).borrow_mut();
	commands.clear();
}

fn assert_command<T: 'static + Debug + PartialEq>(commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>, expected: &T) {
	let commands = commands.clone();
	let mut commands = (&*commands).borrow_mut();
	assert_eq!(commands.len() > 0, true);
	let command = commands.remove(0);
	let command_as_any_box = command.as_any_box();
	let actual = command_as_any_box.downcast_ref::<T>().unwrap();
	assert_eq!(actual, expected)
}

fn setup() -> (Room, Rc<RefCell<Vec<Box<dyn S2CCommand>>>>) {
	let mut room = Room::new();
	let commands = Rc::new(RefCell::new(vec![]));
	let collector = Box::new(S2CCommandCollector::new(commands.clone()));
	room.listener.add_listener(collector);
	(room, commands)
}