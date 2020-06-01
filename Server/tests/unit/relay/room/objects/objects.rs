use cheetah_relay::network::types::hash::HashValue;
use cheetah_relay::room::clients::Client;
use cheetah_relay::room::objects::object::GameObject;
use cheetah_relay::room::objects::owner::Owner;
use cheetah_relay::room::room::Room;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::unit::relay::room::clients::client_stub;

#[test]
fn should_insert_objects() {
    let mut room = setup();
    let object = create_game_object(10);
    let object_id = object.id;
    room.insert_game_object(object);
    assert_eq!(room.objects.get(object_id).is_some(), true)
}


#[test]
fn should_get_objects_by_owner() {
    let mut room = setup();
    let client_a = client_stub(1);
    let client_b = client_stub(2);

    room.insert_game_object(create_game_object_with_client(10, &client_a));
    room.insert_game_object(create_game_object_with_client(55, &client_a));
    room.insert_game_object(create_game_object_with_client(5, &client_b));
    room.insert_game_object(create_game_object_with_client(15, &client_b));

    let objects = room.objects.get_objects_by_owner(Owner::new_owner(&client_a));
    assert_eq!(objects.len(), 2);
    let first_object = objects.first().unwrap().clone();
    let first_object = &*first_object;
    let first_object = first_object.borrow();
    assert_eq!(first_object.id, GameObject::get_global_object_id_by_client(&client_a, 10))
}

fn setup() -> Room {
    Room::new(HashValue::from(""))
}

fn create_game_object(id: u64) -> GameObject {
    GameObject::new(
        id,
        Owner::new_root_owner(),
        AccessGroups::default(),
        GameObjectFields::default(),
    )
}

fn create_game_object_with_client(id: u32, client: &Client) -> GameObject {
    GameObject::new(
        GameObject::get_global_object_id_by_client(client, id),
        Owner::new_owner(client),
        AccessGroups::default(),
        GameObjectFields::default(),
    )
}