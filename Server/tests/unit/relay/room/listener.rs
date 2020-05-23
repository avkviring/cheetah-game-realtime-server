use std::cell::RefCell;
use std::rc::Rc;

use cheetah_relay::room::objects::object::GameObjectTemplate;
use cheetah_relay::room::room::Room;
use crate::unit::relay::room::room::room_stub;
use crate::unit::relay::room::{setup_listener, setup_client};
use crate::unit::relay::room::objects::object::game_object_template_stub;

#[test]
fn should_invoke_on_object_create() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_create 10");
}

#[test]
fn should_invoke_on_object_delete() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &*option.borrow_mut();
	room.delete_game_object(object);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_delete 10");
}

#[test]
fn should_long_counter_change() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_increment_long_counter(object, 5, 50);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_long_counter_change 10 5 50");
}

#[test]
fn should_float_counter_change() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_increment_float_counter(object, 5, 50.5);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_float_counter_change 10 5 50.5");
}

#[test]
fn should_event_fired() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_send_event(object, 5, &vec![1, 2, 3, 4, 5]);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_event_fired 10 5 [1, 2, 3, 4, 5]");
}

#[test]
fn should_structure_updated() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &game_object_template_stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_update_struct(object, 5, &vec![1, 2, 3, 4, 5]);
	assert_eq!((&*invoked).borrow().last().unwrap(), "on_object_struct_updated 10 5 [1, 2, 3, 4, 5]");
}

#[test]
fn should_client_connect() {
	let (mut room, invoked) = setup();
	let client = setup_client(&mut room, "CLIENT_HASH", 0);
	assert_eq!((&*invoked).borrow().last().unwrap(), &format!("on_client_connect {}", client.configuration.id));
}

#[test]
fn should_client_disconnect() {
	let (mut room, invoked) = setup();
	let client = setup_client(&mut room, "CLIENT_HASH", 0);
	room.client_disconnect(&client);
	assert_eq!((&*invoked).borrow().last().unwrap(), &format!("on_client_disconnect {}", client.configuration.id));
}


fn setup() -> (Room, Rc<RefCell<Vec<String>>>) {
	let mut room = room_stub();
	let results = setup_listener(&mut room);
	return (room, results.clone());
}

