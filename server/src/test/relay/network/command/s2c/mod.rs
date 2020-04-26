use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::relay::network::command::s2c::{AffectedClients, S2CCommand};
use crate::relay::network::command::s2c::S2CCommandCollector;
use crate::relay::network::command::s2c::upload_object::UploadObjectS2CCommand;
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::{GameObject, GameObjectTemplate};
use crate::relay::room::objects::owner::Owner;
use crate::relay::room::room::Room;
use crate::test::relay::room::setup_client;

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
	
	assert_command(commands, &UploadObjectS2CCommand {
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
	
	assert_command(commands.clone(), &UploadObjectS2CCommand {
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
	
	assert_command(commands.clone(), &UploadObjectS2CCommand {
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


fn clear_commands(commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>) {
	let commands = commands.clone();
	let mut commands = (&*commands).borrow_mut();
	commands.clear();
}

fn assert_command<T: 'static + Debug + PartialEq>(commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>, expected: &T) {
	let commands = commands.clone();
	let mut commands = (&*commands).borrow_mut();
	let command = commands.remove(0);
	let command_as_any_box = command.as_any_box();
	let actual = command_as_any_box.downcast_ref::<T>();
	assert_eq!(actual.unwrap(), expected)
}

fn setup() -> (Room, Rc<RefCell<Vec<Box<dyn S2CCommand>>>>) {
	let mut room = Room::new();
	let commands = Rc::new(RefCell::new(vec![]));
	let collector = Box::new(S2CCommandCollector::new(commands.clone()));
	room.listener.add_listener(collector);
	(room, commands)
}