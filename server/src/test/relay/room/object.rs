use std::borrow::Borrow;
use std::cell::Ref;
use std::ops::Deref;

use crate::relay::room::clients::ClientConfiguration;
use crate::relay::room::room::Room;
use crate::test::relay::network::StubConnector;

const CLIENT_HASH: &str = "HDJF-OKRD-KDNFK-PWLEO";

#[test]
fn create_game_object() {
    let mut room = setup();
    let id = room.create_client_game_object(16, 255, vec![]);
    assert_eq!(format!("{:x}", id), "10000000ff");
}

fn setup() -> Room {
    let mut room = Room::new();
    room.add_waiting_client(CLIENT_HASH, vec![0, 1, 2, 3, 4]);
    room.connect(CLIENT_HASH.to_string());
    return room;
}
