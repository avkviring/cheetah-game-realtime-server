use crate::relay::room::clients::ClientConfiguration;
use crate::relay::room::room::Room;
use crate::test::relay::network::StubConnector;

const CLIENT_A_HASH: &str = "CLIENT-A-HASH";
const CLIENT_B_HASH: &str = "CLIENT-B-HASH";

/// Коннект клиента, который был заявлен в списке клиентов
#[test]
fn room_client_connect() {
    let (mut room, _) = setup_and_one_client();
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
    let (mut room, _) = setup_and_one_client();
    let result = room.connect("NOT-FOUND-USER-HASH".to_string());
    assert_eq!(result.is_err(), true);
}


#[test]
fn room_client_disconnect() {
    let (mut room, client_id) = setup_and_one_client();
    room.connect(CLIENT_A_HASH.to_string());
    room.disconnect(client_id);
    assert_eq!(room.get_clients().len(), 0);
}

#[test]
fn room_client_disconnect_should_delete_client_object() {
    let (mut room, client_id) = setup_and_one_client();
    room.connect(CLIENT_A_HASH.to_string());
    room.create_client_game_object(client_id, 0, &vec![0]);
    room.disconnect(client_id);
    assert_eq!(room.objects.len(), 0);
}

#[test]
fn create_client_game_object() {
    let (mut room, client_id, _) = setup_and_two_client();
    let id = room.create_client_game_object(client_id, 255, &vec![]).ok().unwrap();
    assert_eq!(format!("{:x}", id), "1000000ff");
    assert_eq!(room.objects.get(id).is_some(), true);
}

#[test]
fn create_client_game_object_should_assign_user_group_if_group_empty() {
    let (mut room, client_id, _) = setup_and_two_client();
    let id = room.create_client_game_object(client_id, 255, &vec![]).ok().unwrap();
    let object = room.objects.get(id).unwrap();
    assert_eq!(object.groups.contains_group(0), true);
    assert_eq!(object.groups.contains_group(5), true);
    assert_eq!(object.groups.contains_group(7), false);
}

#[test]
fn create_client_game_object_when_client_not_found() {
    let (mut room, _, _) = setup_and_two_client();
    let result = room.create_client_game_object(65535, 255, &vec![]);
    assert_eq!(result.is_err(), true)
}

#[test]
fn create_client_game_object_when_missing_client_group() {
    let (mut room, client_id, _) = setup_and_two_client();
    let result = room.create_client_game_object(client_id, 255, &vec![3]);
    assert_eq!(result.is_err(), true)
}


#[test]
fn create_root_game_object() {
    let (mut room, _, _) = setup_and_two_client();
    let id = room.create_root_game_object(255, &vec![]).ok().unwrap();
    assert_eq!(id, 255);
}


fn setup_and_one_client() -> (Room, u16) {
    let client_a_group = vec![0, 1, 2];
    let mut room = Room::new();
    let client_id = room.add_waiting_client(CLIENT_A_HASH, client_a_group);
    (room, client_id)
}

fn setup_and_two_client() -> (Room, u16, u16) {
    let mut room = Room::new();

    let client_a_hash = "CLIENT-A";
    let first_client = room.add_waiting_client(client_a_hash, vec![0, 5]);
    room.connect(client_a_hash.to_string());

    let client_b_hash = "CLIENT-B";
    let second_client = room.add_waiting_client(client_b_hash, vec![0, 5]);
    room.connect(client_b_hash.to_string());

    (room, first_client, second_client)
}
