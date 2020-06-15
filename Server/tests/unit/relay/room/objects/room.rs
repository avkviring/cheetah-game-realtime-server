use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::unit::relay::room::setup_and_two_client;

#[test]
fn should_create_game_object() {
    let (mut room, client, _) = setup_and_two_client();
    let object_id = GameObjectId::new(255, Owner::Root);
    room.new_game_object(
        object_id.clone(),
        AccessGroups::from(0b100000),
        GameObjectFields::default(),
    );
    assert_eq!(true, room.objects.get(&object_id).is_some());
}

#[test]
fn should_error_when_create_exists_game_object() {
    let (mut room, client, _) = setup_and_two_client();
    let object_id = GameObjectId::new(255, Owner::Root);
    assert!(matches!(room.new_game_object(object_id.clone(),AccessGroups::from(0b100000),GameObjectFields::default(),), Result::Ok(_)));
    assert!(matches!(room.new_game_object(object_id,AccessGroups::from(0b100000),GameObjectFields::default(),), Result::Err(_)));
}


#[test]
fn delete_client_game_object() {
    let (mut room, client, _) = setup_and_two_client();
    let object_id = GameObjectId::new(255, Owner::Root);
    room.new_game_object(
        object_id.clone(),
        AccessGroups::from(0b100000),
        GameObjectFields::default(),
    );
    let game_object = room.objects.get(&object_id).unwrap();
    room.delete_game_object(&(*(*(game_object.clone())).borrow()));
    assert_eq!(room.objects.get(&object_id).is_none(), true);
}

