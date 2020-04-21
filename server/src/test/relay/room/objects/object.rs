use std::cmp::Ordering;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::{GameObject, FieldID, GameObjectTemplate};

#[test]
fn should_store_struct_data_in_game_object() {
	let mut object = setup();
	let struct_id: u16 = 10;
	object.update_struct(struct_id, vec![1 as u8, 2 as u8, 3 as u8]);
	let data = object.get_struct(struct_id);
	assert_eq!(data.is_some(), true);
	let bytes = data.unwrap();
	assert_eq!(match bytes.cmp(&vec![1 as u8, 2 as u8, 3 as u8]) {
		Ordering::Less => { false }
		Ordering::Equal => { true }
		Ordering::Greater => { false }
	}, true)
}

#[test]
fn should_error_when_struct_not_found_in_game_object() {
	let object = setup();
	let struct_id: u16 = 10;
	let data = object.get_struct(struct_id);
	assert_eq!(data.is_none(), true);
}


#[test]
fn test_long_counter() {
	let mut object = setup();
	let field_id: FieldID = 10;
	object.set_long_counter(field_id, 100);
	let count1 = object.increment_long_counter(field_id, 5);
	let count2 = object.get_long_counter(field_id);
	assert_eq!(count1, 105);
	assert_eq!(count1, count2);
}

#[test]
fn test_float_counter() {
	let mut object = setup();
	let field_id:FieldID = 10;
	object.set_float_counter(field_id, 100.0);
	let count1 = object.increment_float_counter(field_id, 5.0);
	let count2 = object.get_float_counter(field_id);
	assert_eq!(count1, 105.0);
	assert_eq!(count1, count2);
}


fn setup() -> GameObject {
	GameObject::new_client_object(&Client::stub(0), 0, &GameObjectTemplate::stub())
}

