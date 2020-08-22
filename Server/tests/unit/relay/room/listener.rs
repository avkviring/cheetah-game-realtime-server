use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay::room::objects::object::{GameObject};
use cheetah_relay::room::Room;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::unit::relay::room::{setup_client, setup_listener};
use crate::unit::relay::room::room::room_stub;
use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};

#[test]
fn should_invoke_on_object_create() {
	let (_room, invoked, _object) = setup();
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_create 10");
}

#[test]
fn should_invoke_on_object_delete() {
	let (mut room, invoked, object) = setup();
	let object = &*object.borrow_mut();
	room.delete_game_object(object);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_delete 10");
}

#[test]
fn should_long_counter_change() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_increment_long_counter(object, 5, 50);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_long_counter_change 10 5 50");
}

#[test]
fn should_float_counter_change() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_increment_float_counter(object, 5, 50.5);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_float_counter_change 10 5 50.5");
}

#[test]
fn should_float_counter_set() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_set_float_counter(object, 5, 50.5);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_float_counter_set 10 5 50.5");
}

#[test]
fn should_long_counter_set() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_set_long_counter(object, 5, 50);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_long_counter_set 10 5 50");
}

#[test]
fn should_event_fired() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_send_event(object, 5, &vec![1, 2, 3, 4, 5]);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_event_fired 10 5 [1, 2, 3, 4, 5]");
}

#[test]
fn should_structure_updated() {
	let (mut room, invoked, object) = setup();
	let object = &mut *object.borrow_mut();
	room.object_update_struct(object, 5, vec![1, 2, 3, 4, 5]);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_struct_updated 10 5 [1, 2, 3, 4, 5]");
}

#[test]
fn should_client_connect() {
	let (mut room, invoked, _) = setup();
	let client = setup_client(&mut room, "CLIENT_HASH", 0);
	assert_eq!((&*invoked).borrow().last().unwrap(), &format!("on_client_connect {}", client.configuration.id));
}

#[test]
fn should_client_disconnect() {
	let (mut room, invoked, _) = setup();
	let client = setup_client(&mut room, "CLIENT_HASH", 0);
	room.client_disconnect(&client);
	assert_eq!((&*invoked).borrow().last().unwrap(), &format!("on_client_disconnect {}", client.configuration.id));
}


fn setup() -> (Room, Rc<RefCell<Vec<String>>>, Rc<RefCell<GameObject>>) {
	let mut room = room_stub();
	let results = setup_listener(&mut room);
	let object_id = ServerGameObjectId::new(10, ServerOwner::Root);
	room.new_game_object(
		object_id.clone(),
		AccessGroups::default(),
		GameObjectFields::default(),
	).expect("error create game object");
	let object = room.objects.get(&object_id).unwrap();
	(room, results, object)
}

