/// Тесты room cвязанные с коннектом клиента

use crate::relay::room::groups::AccessGroups;
use crate::relay::room::objects::object::GameObjectTemplate;
use crate::relay::room::room::Room;
use crate::relay::network::types::hash::{ToHashValue, HashValue};

const CLIENT_HASH: &str = "12312313212";

/// Коннект клиента, который был заявлен в списке клиентов
#[test]
fn room_client_connect() {
	let mut room = Room::stub();
	
	let client_hash = HashValue::from(CLIENT_HASH);
	room.add_client_to_waiting_list(&client_hash, AccessGroups::from(0b1));
	assert_eq!(room.clients.waiting_clients.len(), 1);
	
	let result = room.client_connect(&client_hash);
	assert_eq!(result.is_ok(), true);
	assert_eq!(room.clients.waiting_clients.len(), 0);
	assert_eq!(room.clients.get_clients().len(), 1);
	
	let groups = &room.clients.get_clients().last().unwrap().configuration.groups;
	assert_eq!(groups.contains_group(0), true);
	assert_eq!(groups.contains_group(3), false);
}


/// Коннект клиента, который не был заявлен в списке клиентов
#[test]
fn room_client_connect_when_client_not_found() {
	let mut room = Room::stub();
	let result = room.client_connect(&"NOT-FOUND-USER-HASH".to_hash_value());
	assert_eq!(result.is_err(), true);
}

#[test]
fn room_client_disconnect() {
	let mut room = Room::stub();
	room.add_client_to_waiting_list(&CLIENT_HASH.to_hash_value(), AccessGroups::new());
	let client = room.client_connect(&CLIENT_HASH.to_hash_value());
	room.client_disconnect(&client.ok().unwrap().clone());
	assert_eq!(room.clients.clients.is_empty(), true);
}


#[test]
fn room_client_disconnect_should_delete_client_object() {
	let mut room = Room::stub();
	room.add_client_to_waiting_list(&CLIENT_HASH.to_hash_value(), AccessGroups::new());
	let connect_result = room.client_connect(&CLIENT_HASH.to_hash_value());
	let client = connect_result.ok().unwrap();
	room.create_client_game_object(&client.clone(), 0, &GameObjectTemplate::stub());
	room.client_disconnect(&client.clone());
	assert_eq!(room.objects.len(), 0);
}