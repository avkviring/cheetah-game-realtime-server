use std::borrow::Borrow;
use std::cell::Ref;
use std::ops::Deref;

use crate::relay::room::clients::ClientConfiguration;
use crate::relay::room::room::Room;
use crate::test::relay::network::StubConnector;

#[test]
fn create_game_object() {
    let (mut room, client_id) = setup();
    let id = room.create_client_game_object(client_id, 255, vec![]);
    assert_eq!(format!("{:x}", id), "2000000ff");
    assert_eq!(room.objects.get(id).is_some(), true);
}

#[test]
fn create_game_object_should_assign_user_group_if_group_empty() {
    let (mut room, client_id) = setup();
    let id = room.create_client_game_object(client_id, 255, vec![]);
    let object = room.objects.get(id).unwrap();
    assert_eq!(object.groups.contains_group(0), true);
    assert_eq!(object.groups.contains_group(5), true);
    assert_eq!(object.groups.contains_group(7), false);
}


fn setup() -> (Room, u16) {
    let mut room = Room::new();

    let client_a_hash = "CLIENT-A";
    room.add_waiting_client(client_a_hash, vec![0, 5]);
    room.connect(client_a_hash.to_string());

    let client_b_hash = "CLIENT-B";
    let second_client = room.add_waiting_client(client_b_hash, vec![0, 5]);
    room.connect(client_b_hash.to_string());

    return (room, second_client);
}
