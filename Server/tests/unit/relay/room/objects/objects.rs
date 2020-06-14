use cheetah_relay::room::clients::Client;
use cheetah_relay::room::objects::object::GameObject;
use cheetah_relay::room::Room;
use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::unit::relay::room::clients::client_stub;

#[test]
fn should_insert_objects() {
	let mut room = setup();
	let object = create_game_object(10);
	let object_id = object.id.clone();
	room.insert_game_object(object);
	assert_eq!(room.objects.get(&object_id).is_some(), true)
}


#[test]
fn should_get_objects_by_owner() {
	let mut room = setup();
	let client_a = client_stub(1);
	let client_b = client_stub(2);
	
	room.insert_game_object(create_game_object_with_client(10, &client_a));
	room.insert_game_object(create_game_object_with_client(55, &client_a));
	room.insert_game_object(create_game_object_with_client(5, &client_b));
	room.insert_game_object(create_game_object_with_client(15, &client_b));
	
	let objects = room.objects.get_objects_by_owner(Owner::Client(client_a.configuration.id));
	assert_eq!(objects.len(), 2);
	let first_object = objects.first().unwrap().clone();
	let first_object = &*first_object;
	let first_object = first_object.borrow();
	assert_eq!(first_object.id.id, 10);
	assert!(matches!(first_object.id.owner, Owner::Client(client_id) if client_id==client_a.configuration.id))
}

fn setup() -> Room {
	Room::new(HashValue::from(""))
}

fn create_game_object(id: u32) -> GameObject {
	GameObject::new(
		GameObjectId::new(id, Owner::Root),
		AccessGroups::default(),
		GameObjectFields::default(),
	)
}

fn create_game_object_with_client(id: u32, client: &Client) -> GameObject {
	GameObject::new(
		GameObjectId::new(id, Owner::Client(client.configuration.id)),
		AccessGroups::default(),
		GameObjectFields::default(),
	)
}