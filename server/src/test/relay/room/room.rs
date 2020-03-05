use crate::relay::room::clients::ClientConfiguration;
use crate::relay::room::room::Room;
use crate::test::relay::network::StubConnector;

const CLIENT_A_HASH: &str = "CLIENT-A-HASH";
const CLIENT_B_HASH: &str = "CLIENT-B-HASH";

/// Коннект клиента, который был заявлен в списке клиентов
#[test]
fn room_client_connect() {
    let mut room = setup();
    let result = room.connect(CLIENT_A_HASH.to_string());

    assert_eq!(result.is_ok(), true);
    assert_eq!(room.waiting_clients.len(), 0);
    assert_eq!(room.get_clients().len(), 1);

    let groups = &room.get_clients().last().unwrap().configuration.groups;
    assert_eq!(groups.contains_group(0), true);
    assert_eq!(groups.contains_group(3), false);
}

/// Коннект клиента, который не был заявлен в списке клиентов
#[test]
fn room_client_connect_when_client_not_found() {
    let mut room = setup();
    let result = room.connect("NOT-FOUND-USER-HASH".to_string());
    assert_eq!(result.is_err(), true);
}


fn setup() -> Room {
    let client_a_group = vec![0, 1, 2];
    let mut room = Room::new();
    room.add_waiting_client(CLIENT_A_HASH, client_a_group);
    room
}