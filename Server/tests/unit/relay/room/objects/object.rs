use std::cmp::Ordering;

use cheetah_relay::network::types::hash::HashValue;
use cheetah_relay::room::objects::object::GameObject;
use cheetah_relay::room::objects::owner::Owner;
use cheetah_relay::room::room::Room;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

#[test]
fn should_store_struct_data_in_game_object() {
    let (mut room, mut object) = setup();
    let struct_id: u16 = 10;
    room.object_update_struct(&mut object, struct_id, vec![1 as u8, 2 as u8, 3 as u8]);
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
    let (_, mut object) = setup();
    let struct_id: u16 = 10;
    let data = object.get_struct(struct_id);
    assert_eq!(data.is_none(), true);
}


#[test]
fn test_long_counter() {
    let (mut room, mut object) = setup();
    let field_id: FieldID = 10;
    room.object_set_long_counter(&mut object, field_id, 100);
    let count1 = room.object_increment_long_counter(&mut object, field_id, 5);
    let count2 = object.get_long_counter(field_id);
    assert_eq!(count1, 105);
    assert_eq!(count1, count2);
}

#[test]
fn test_float_counter() {
    let (mut room, mut object) = setup();
    let field_id: FieldID = 10;
    room.object_set_float_counter(&mut object, field_id, 100.0);
    let count1 = room.object_increment_float_counter(&mut object, field_id, 5.0);
    let count2 = object.get_float_counter(field_id);
    assert_eq!(count1 as u64, 105);
    assert_eq!(count1 as u64, count2 as u64);
}


fn setup() -> (Room, GameObject) {
    let object = GameObject::new(
        0,
        Owner::new_root_owner(),
        AccessGroups::default(),
        GameObjectFields::default());
    let room = Room::new(HashValue::from(""));
    (room, object)
}


