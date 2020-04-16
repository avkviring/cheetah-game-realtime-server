use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::Objects;
use crate::relay::room::objects::object::GameObject;
use crate::relay::room::objects::owner::Owner;

#[test]
fn should_insert_objects() {
	let mut objects = setup_game_objects();
	let object = GameObject::new_client_object(&Client::stub(0), 10, AccessGroups::new());
	let object_id = object.id;
	objects.insert(object);
	assert_eq!(objects.get(object_id).is_some(), true)
}


#[test]
fn should_delete_objects_by_owner() {
	let mut objects = setup_game_objects();
	let client_a = Client::stub(1);
	let client_b = Client::stub(2);
	objects.insert(GameObject::new_client_object(&client_a, 10, AccessGroups::new()));
	objects.insert(GameObject::new_client_object(&client_b, 10, AccessGroups::new()));
	assert_eq!(objects.len(), 2);
	objects.delete_objects_by_owner(Owner::new_owner(&client_a));
	assert_eq!(objects.len(), 1);
}

fn setup_game_objects() -> Objects {
	let objects = Default::default();
	return objects;
}