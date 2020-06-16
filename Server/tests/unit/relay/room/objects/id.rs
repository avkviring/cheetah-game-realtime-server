use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};
use cheetah_relay_common::room::owner::ClientOwner;

#[test]
fn should_convert_with_owner_root() {
	let server_game_object_id = ServerGameObjectId::new(100, ServerOwner::Root);
	let client_object_id = server_game_object_id.to_client_object_id(Option::None);
	assert_eq!(server_game_object_id, ServerGameObjectId::from_client_object_id(Option::None, &client_object_id));
}

#[test]
fn should_convert_with_current_client() {
	let server_game_object_id = ServerGameObjectId::new(100, ServerOwner::Client(100));
	let client_object_id = server_game_object_id.to_client_object_id(Option::Some(100));
	assert!(matches!(client_object_id.owner, ClientOwner::CurrentClient));
	assert_eq!(server_game_object_id, ServerGameObjectId::from_client_object_id(Option::Some(100), &client_object_id));
}

#[test]
fn should_convert_with_client() {
	let server_game_object_id = ServerGameObjectId::new(100, ServerOwner::Client(100));
	let client_object_id = server_game_object_id.to_client_object_id(Option::Some(200));
	assert!(matches!(client_object_id.owner, ClientOwner::Client(client) if client == 100));
	assert_eq!(server_game_object_id, ServerGameObjectId::from_client_object_id(Option::Some(200), &client_object_id));
}