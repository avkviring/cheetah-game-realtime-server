use crate::relay::room::room::Room;
use crate::relay::room::structs::UserAuth;
use crate::test::relay::network::StubConnector;

/// Коннект пользователя, который был заявлен в списке пользователей
#[test]
fn room_user_connect<'a>() {
    let hashA = "HDJF-OKRD-KDNFK-PWLEO";
    let mut room = Room::new(vec![UserAuth::new(hashA)]);
    let mut connector = StubConnector {};
    let result = room.connect(hashA.to_string(), &mut connector);

    assert_eq!(result.is_ok(), true);
    assert_eq!(room.waiting_users.len(), 0);
    assert_eq!(room.users.len(), 1);
}


/// Коннект пользователя, который не был заявлен в списке пользователей
#[test]
fn room_user_connect_when_user_not_found<'a>() {
    let hashA = "HDJF-OKRD-KDNFK-PWLEO";
    let mut room = Room::new(vec![UserAuth::new(hashA)]);
    let mut connector = StubConnector {};
    let result = room.connect("NOT-FOUND-USER-HASH".to_string(), &mut connector);
    assert_eq!(result.is_err(), true);
}


