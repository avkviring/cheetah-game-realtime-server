use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::object::{GameObject, GameObjectTemplate};
use crate::relay::room::room::Room;

#[test]
fn should_invoke_on_object_create() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	assert_eq!(invoked.replace("".to_string()), "on_object_create 10");
}

#[test]
fn should_invoke_on_object_delete() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &*option.borrow_mut();
	room.delete_game_object(object);
	assert_eq!(invoked.replace("".to_string()), "on_object_delete 10");
}

#[test]
fn should_long_counter_change() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_increment_long_counter(object, 5, 50);
	
	assert_eq!(invoked.replace("".to_string()), "on_object_long_counter_change 10 5 50");
}

#[test]
fn should_float_counter_change() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_increment_float_counter(object, 5, 50.5);
	assert_eq!(invoked.replace("".to_string()), "on_object_float_counter_change 10 5 50.5");
}

#[test]
fn should_event_fired() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_send_event(object, 5, &vec![1, 2, 3, 4, 5]);
	assert_eq!(invoked.replace("".to_string()), "on_object_event_fired 10 5 [1, 2, 3, 4, 5]");
}

#[test]
fn should_structure_updated() {
	let (mut room, invoked) = setup();
	room.create_root_game_object(10, &GameObjectTemplate::stub());
	let option = room.objects.get(10).unwrap().clone();
	let object = &mut *option.borrow_mut();
	room.object_update_struct(object, 5, &vec![1, 2, 3, 4, 5]);
	assert_eq!(invoked.replace("".to_string()), "on_object_struct_updated 10 5 [1, 2, 3, 4, 5]");
}


fn setup() -> (Room, Rc<Cell<String>>) {
	let mut room = Room::new();
	let mut invoked = Rc::new(Cell::new("".to_string()));
	let mut listener = Box::new(TestListener {
		invoked: invoked.clone()
	});
	room.listener.add_listener(listener);
	return (room, invoked);
}


struct TestListener {
	pub invoked: Rc<Cell<String>>
}

impl RoomListener for TestListener {
	fn on_object_created(&mut self, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_create {}", game_object.id));
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_delete {}", game_object.id));
	}
	
	fn on_object_long_counter_change(&mut self, field_id: u16, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_long_counter_change {} {} {}", game_object.id, field_id, game_object.get_long_counter(field_id)));
	}
	
	fn on_object_float_counter_change(&mut self, field_id: u16, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_float_counter_change {} {} {}", game_object.id, field_id, game_object.get_float_counter(field_id)));
	}
	
	fn on_object_event_fired(&mut self, field_id: u16, event_data: &Vec<u8>, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_event_fired {} {} {:?}", game_object.id, field_id, event_data));
	}
	
	fn on_object_struct_updated(&mut self, field_id: u16, game_object: &GameObject) {
		let rc = self.invoked.clone();
		rc.set(format!("on_object_struct_updated {} {} {:?}", game_object.id, field_id, game_object.get_struct(field_id).unwrap()));
	}
}