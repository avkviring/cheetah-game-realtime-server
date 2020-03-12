use std::borrow::Borrow;
use std::cell::Ref;
use std::cmp::Ordering;
use std::ops::Deref;

use bytes::Bytes;

use crate::relay::room::clients::ClientConfiguration;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::object::GameObject;
use crate::relay::room::room::Room;

#[test]
fn should_store_struct_data_in_game_object() {
    let mut object = setup();
    let struct_id: u16 = 10;
    object.update_struct(struct_id, &[1, 2, 3]);
    let data = object.get_struct(struct_id);
    assert_eq!(data.is_some(), true);
    let bytes = data.unwrap();
    assert_eq!(match bytes.cmp(&Bytes::copy_from_slice(&[1, 2, 3])) {
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
fn should_update_counter() {
    let mut object = setup();
    let counter_id: u16 = 10;
    object.set_counter(counter_id, 100);
    object.increment_counter(counter_id, 5);
    let count = object.get_counter(counter_id);
    assert_eq!(count, 105);
}


fn setup() -> GameObject {
    GameObject::new(0, 0, AccessGroups::new())
}

