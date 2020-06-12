use cheetah_relay::room::Room;
use cheetah_relay_common::room::access::AccessGroups;
use crate::unit::relay::room::{setup_and_two_client, setup_client, setup_listener};
use cheetah_relay_common::network::hash::HashValue;

#[test]
fn should_load_game_objects_when_new_client_connected() {
    let (mut room, client_a, _client_b) = setup_and_two_client();
    room.create_client_game_object(
        &*client_a,
        10,
        AccessGroups::from(0b1),
        Default::default(),
    );
    room.create_client_game_object(
        &*client_a,
        20,
        AccessGroups::from(0b1),
        Default::default(),
    );


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