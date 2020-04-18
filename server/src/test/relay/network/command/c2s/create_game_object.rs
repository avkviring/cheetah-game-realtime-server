use std::borrow::Borrow;

use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::network::command::c2s::create_game_object::CreateGameObjectC2SCommand;
use crate::relay::room::objects::object::GameObject;
use crate::test::relay::room::setup_and_two_client;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	buffer.write_u8(2);
	buffer.write_u8(10);
	buffer.write_u8(20);
	
	let result = CreateGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<CreateGameObjectC2SCommand>().unwrap();
	
	assert_eq!(command.local_id, 100);
	assert_eq!(command.groups, vec![10 as u8, 20 as u8])
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	buffer.write_u8(2);
	buffer.write_u8(10);
	
	let result = CreateGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let command = CreateGameObjectC2SCommand {
		local_id: 0,
		groups: vec![],
	};
	command.execute(&client.clone(), &mut room);
	let global_object_id = GameObject::to_global_object_id(client.borrow(), 0);
	assert_eq!(room.objects.get(global_object_id).is_some(), true);
}