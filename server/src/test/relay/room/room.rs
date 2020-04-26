use crate::relay::room::objects::object::GameObjectTemplate;
use crate::test::relay::room::{setup_and_two_client, setup_client, setup_listener};

#[test]
fn should_load_game_objects_when_new_client_connected() {
	let (mut room, client_a, client_b) = setup_and_two_client();
	room.create_client_game_object(&*client_a, 10, &GameObjectTemplate::stub_with_group(0b1));
	room.create_client_game_object(&*client_a, 20, &GameObjectTemplate::stub_with_group(0b1));
	
	
	let listener = setup_listener(&mut room);
	setup_client(&mut room, "CLIENT_C", 0b1);
	
	
	
}