use cheetah_relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;
use cheetah_relay::room::objects::object::GameObjectTemplate;
use cheetah_relay::network::types::niobuffer::NioBuffer;
use crate::unit::relay::room::setup_and_two_client;
use crate::unit::relay::room::objects::object::game_object_template_stub_with_group;

#[test]
fn should_decode() {
	let mut buffer = NioBuffer::new();
	buffer.write_u64(100);
	buffer.flip();
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), true);
	let command = result.unwrap();
	
	assert_eq!(command.global_object_id, 100)
}

#[test]
fn should_not_decode_when_data_not_enough() {
	let mut buffer = NioBuffer::new();
	buffer.write_u32(100);
	buffer.flip();
	let result = DeleteGameObjectC2SCommand::decode(&mut buffer);
	assert_eq!(result.is_some(), false);
}

#[test]
fn test_execute_command() {
	let (mut room, client, _) = setup_and_two_client();
	let global_object_id = room.create_client_game_object(
		&client.clone(),
		0,
		&game_object_template_stub_with_group(0b100000),
	).ok().unwrap();
	
	let command = DeleteGameObjectC2SCommand {
		global_object_id,
	};
	command.execute(&client.clone(), &mut room);
	
	assert_eq!(room.objects.get(global_object_id).is_none(), true);
}