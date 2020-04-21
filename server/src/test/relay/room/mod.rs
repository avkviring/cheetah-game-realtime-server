use crate::relay::room::room::Room;
use std::rc::Rc;
use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;

mod room;
mod objects;
mod clients;
mod groups;

pub fn setup_and_two_client() -> (Room, Rc<Client>, Rc<Client>) {
	let mut room = Room::new();
	
	let client_a_hash = "CLIENT-A";
	room.add_client_to_waiting_list(client_a_hash.to_string(), AccessGroups::from(0b100001));
	let first_client = room.client_connect(client_a_hash.to_string()).unwrap();
	
	let client_b_hash = "CLIENT-B";
	room.add_client_to_waiting_list(client_b_hash.to_string(), AccessGroups::from(0b100001));
	let second_client = room.client_connect(client_b_hash.to_string()).unwrap();
	
	(room, first_client, second_client)
}


