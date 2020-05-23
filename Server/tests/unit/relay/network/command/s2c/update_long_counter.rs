use cheetah_relay::network::command::s2c::S2CCommand;
use cheetah_relay::network::command::s2c::update_long_counter::UpdateLongCounterS2CCommand;
use crate::unit::relay::network::command::s2c::create_buffer_with_capacity;

#[test]
fn should_true_when_buffer_is_enough() {
	let mut buffer = create_buffer_with_capacity(8 + 2 + 8);
	assert_eq!(create_command().encode(&mut buffer), true)
}

#[test]
fn should_false_when_buffer_for_write_is_small_1() {
	let mut buffer = create_buffer_with_capacity(0);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_2() {
	let mut buffer = create_buffer_with_capacity(8);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_3() {
	let mut buffer = create_buffer_with_capacity(8 + 2);
	assert_eq!(create_command().encode(&mut buffer), false)
}

#[test]
fn should_false_when_buffer_for_write_is_small_4() {
	let mut buffer = create_buffer_with_capacity(8 + 2 + 7);
	assert_eq!(create_command().encode(&mut buffer), false)
}


fn create_command() -> UpdateLongCounterS2CCommand {
	UpdateLongCounterS2CCommand {
		global_object_id: Default::default(),
		field_id: Default::default(),
		value: Default::default(),
	}
}