use cheetah_relay::room::clients::Client;
use cheetah_relay::room::objects::Objects;
use cheetah_relay::room::objects::owner::Owner;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use crate::unit::relay::room::clients::client_stub;
use cheetah_relay::room::objects::object::GameObject;

#[test]
fn should_insert_objects() {
    let mut objects = setup_game_objects();
    let object = GameObject::new_client_object(&client_stub(0), 10, AccessGroups::default(), GameObjectFields::default());
    let object_id = object.id;
    objects.insert(object);
    assert_eq!(objects.get(object_id).is_some(), true)
}


#[test]
fn should_get_objects_by_owner() {
    let mut objects = setup_game_objects();
    let client_a = client_stub(1);
    let client_b = client_stub(2);
    objects.insert(GameObject::new_client_object(&client_a, 10, AccessGroups::default(), GameObjectFields::default()));
    objects.insert(GameObject::new_client_object(&client_a, 55, AccessGroups::default(), GameObjectFields::default()));
    objects.insert(GameObject::new_client_object(&client_b, 5, AccessGroups::default(), GameObjectFields::default()));
    objects.insert(GameObject::new_client_object(&client_b, 15, AccessGroups::default(), GameObjectFields::default()));
    let objects = objects.get_objects_by_owner(Owner::new_owner(&client_a));
    assert_eq!(objects.len(), 2);
    let first_object = objects.first().unwrap().clone();
    let first_object = &*first_object;
    let first_object = first_object.borrow();
    assert_eq!(first_object.id, GameObject::get_global_object_id_by_client(&client_a, 10))
}

fn setup_game_objects() -> Objects {
    Default::default()
}