use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::{C2SCommandDecoder, C2SCommandExecutor};
use crate::relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;
use crate::test::relay::room::setup_and_two_client;
use crate::relay::room::objects::object::GameObjectTemplate;

#[test]
fn should_decode() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u64(100);
	
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	
	let result = &*(result.unwrap());
	let command = result.as_any_ref().downcast_ref::<DeleteGameObjectC2SCommand>().unwrap();
	
	assert_eq!(command.global_object_id, 100)
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = ByteBuffer::new();
	buffer.write_u32(100);
	
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(
		&client.clone(),
		0,
		&GameObjectTemplate::stub_with_group(0b100000)
	).ok().unwrap();
	
	let command = DeleteGameObjectC2SCommand {
		global_object_id,
	};
	command.execute(&client.clone(), &mut room);
	
	assert_eq!(room.objects.get(global_object_id).is_none(), true);
}