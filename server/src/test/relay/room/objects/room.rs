use std::rc::Rc;

use crate::relay::room::clients::Client;
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::room::Room;
use crate::test::relay::room::setup_and_two_client;

#[test]
fn create_client_game_object() {
	let (mut room, client, _) = setup_and_two_client();
	let id = room.create_client_game_object(&client.clone(), 255, Option::None).ok().unwrap();
	assert_eq!(format!("{:x}", id), "1000000ff");
	assert_eq!(room.objects.get(id).is_some(), true);
}


#[test]
fn delete_client_game_object() {
	let (mut room, client, _) = setup_and_two_client();
	let id = room.create_client_game_object(&client.clone(), 255, Option::None).ok().unwrap();
	let game_object = room.objects.get_by_owner(&client.clone(), 255).unwrap();
	room.delete_game_object(&(*(*(game_object.clone())).borrow()));
	assert_eq!(room.objects.get(id).is_none(), true);
}


#[test]
fn create_client_game_object_should_assign_user_group_if_group_empty() {
	let (mut room, client, _) = setup_and_two_client();
	let id = room.create_client_game_object(&client.clone(), 255, Option::None).ok().unwrap();
	let object_rc = room.objects.get(id).unwrap();
	let groups = &object_rc.borrow().groups;
	assert_eq!(groups.contains_group(0), true);
	assert_eq!(groups.contains_group(5), true);
	assert_eq!(groups.contains_group(7), false);
}


#[test]
fn create_client_game_object_when_missing_client_group() {
	let (mut room, client, _) = setup_and_two_client();
	let result = room
		.create_client_game_object(
			&client.clone(),
			255,
			Option::Some(AccessGroups::from(vec![3])),
		);
	assert_eq!(result.is_err(), true)
}


#[test]
fn create_root_game_object() {
	let (mut room, _, _) = setup_and_two_client();
	let id = room.create_root_game_object(255, AccessGroups::new()).ok().unwrap();
	assert_eq!(id, 255);
}