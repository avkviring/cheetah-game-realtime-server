use crate::relay::network::command::s2c::delete_game_object::DeleteGameObjectS2CCommand;
use crate::relay::network::command::s2c::S2CCommand;
use crate::test::unit::relay::network::command::s2c::create_buffer_with_capacity;

#[test]
fn should_true_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(8);
	assert_eq!(create_command().encode(&mut buffer), true)
}

#[test]
fn should_false_when_buffer_for_write_is_small() {
	let mut buffer = create_buffer_with_capacity(7);
	assert_eq!(create_command().encode(&mut buffer), false)
}

fn create_command() -> DeleteGameObjectS2CCommand {
	DeleteGameObjectS2CCommand {
		global_object_id: Default::default(),
	}
}