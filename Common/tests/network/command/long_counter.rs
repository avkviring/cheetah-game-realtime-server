use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_set_long_counter_command() {
	let structure = SetLongCounterCommand {
		object_id: GameObjectId::new(std::u32::MAX, Owner::Root),
		field_id: std::u16::MAX,
		value: 200,
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}

#[test]
fn test_codec_for_increment_long_counter_command() {
	let structure = IncrementLongCounterC2SCommand {
		object_id: GameObjectId::new(std::u32::MAX, Owner::Root),
		field_id: std::u16::MAX,
		increment: 200,
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}