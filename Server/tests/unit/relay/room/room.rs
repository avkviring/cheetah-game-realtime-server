use cheetah_relay::network::types::hash::HashValue;
use cheetah_relay::room::objects::object::GameObjectTemplate;
use cheetah_relay::room::room::Room;

use crate::unit::relay::room::{setup_and_two_client, setup_client, setup_listener};
use crate::unit::relay::room::objects::object::game_object_template_stub_with_group;

#[test]
fn should_load_game_objects_when_new_client_connected() {
	let (mut room, client_a, _client_b) = setup_and_two_client();
	room.create_client_game_object(&*client_a, 10, &game_object_template_stub_with_group(0b1));
	room.create_client_game_object(&*client_a, 20, &game_object_template_stub_with_group(0b1));
	
	
	let listener = setup_listener(&mut room);
	setup_client(&mut room, "CLIENT_C", 0b1);
	let listener = &*listener.clone();
	let listener = listener.borrow();
	println!("listener {:?}", listener);
}

pub fn hash_value_stub() -> HashValue {
	HashValue::from("room_hash")
}

pub fn room_stub() -> Room {
	Room {
		hash: hash_value_stub(),
		listener: Default::default(),
		clients: Default::default(),
		objects: Default::default(),
	}
}