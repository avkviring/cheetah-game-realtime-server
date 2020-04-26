use std::cell::{ RefCell};
use std::rc::Rc;

use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::object::{GameObject, GroupType};
use crate::relay::room::objects::Objects;
use crate::relay::room::room::Room;

mod room;
mod objects;
mod clients;
mod groups;
mod listener;

pub fn setup_and_two_client() -> (Room, Rc<Client>, Rc<Client>) {
	let mut room = Room::new();
	let first_client = setup_client(&mut room, "CLIENT-A", 0b100001);
	let second_client = setup_client(&mut room, "CLIENT-B", 0b100001);
	(room, first_client, second_client)
}

pub fn setup_client(room: &mut Room, client_hash: &str, group: GroupType) -> Rc<Client> {
	room.add_client_to_waiting_list(client_hash.to_string(), AccessGroups::from(group));
	let second_client = room.client_connect(client_hash.to_string()).unwrap();
	second_client
}

pub fn setup_listener(room: &mut Room) -> Rc<RefCell<Vec<String>>> {
	let mut results = Rc::new(RefCell::new(Vec::<String>::new()));
	let mut listener = Box::new(TestListener { results: results.clone() });
	room.listener.add_listener(listener);
	results
}


struct TestListener {
	pub results: Rc<RefCell<Vec<String>>>
}

impl RoomListener for TestListener {
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_create {}", game_object.id));
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_delete {}", game_object.id));
	}
	
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_client_connect {}", client.configuration.id));
	}
	
	fn on_client_disconnect(&mut self, client: &Client) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_client_disconnect {}", client.configuration.id));
	}
	
	fn on_object_long_counter_change(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_long_counter_change {} {} {}", game_object.id, field_id, game_object.get_long_counter(field_id)));
	}
	
	fn on_object_float_counter_change(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_float_counter_change {} {} {}", game_object.id, field_id, game_object.get_float_counter(field_id)));
	}
	
	fn on_object_event_fired(&mut self, field_id: u16, event_data: &Vec<u8>, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_event_fired {} {} {:?}", game_object.id, field_id, event_data));
	}
	
	fn on_object_struct_updated(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		let mut rc = self.results.clone();
		rc.borrow_mut().push(format!("on_object_struct_updated {} {} {:?}", game_object.id, field_id, game_object.get_struct(field_id).unwrap()));
	}
}