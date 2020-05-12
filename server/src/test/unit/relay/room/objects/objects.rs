use crate::relay::room::clients::Client;
use crate::relay::room::objects::object::{GameObject, GameObjectTemplate};
use crate::relay::room::objects::Objects;
use crate::relay::room::objects::owner::Owner;

#[test]
fn should_insert_objects() {
	let mut objects = setup_game_objects();
	let object = GameObject::new_client_object(&Client::stub(0), 10, &GameObjectTemplate::stub());
	let object_id = object.id;
	objects.insert(object);
	assert_eq!(objects.get(object_id).is_some(), true)
}


#[test]
fn should_get_objects_by_owner() {
	let mut objects = setup_game_objects();
	let client_a = Client::stub(1);
	let client_b = Client::stub(2);
	objects.insert(GameObject::new_client_object(&client_a, 10, &GameObjectTemplate::stub()));
	objects.insert(GameObject::new_client_object(&client_a, 55, &GameObjectTemplate::stub()));
	objects.insert(GameObject::new_client_object(&client_b, 5, &GameObjectTemplate::stub()));
	objects.insert(GameObject::new_client_object(&client_b, 15, &GameObjectTemplate::stub()));
	let objects = objects.get_objects_by_owner(Owner::new_owner(&client_a));
	assert_eq!(objects.len(), 2);
	let first_object = objects.first().unwrap().clone();
	let first_object = &*first_object;
	let first_object = first_object.borrow();
	assert_eq!(first_object.id, GameObject::get_global_object_id_by_client(&client_a, 10))
}

fn setup_game_objects() -> Objects {
	return Default::default();
}