use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_event_command() {
	let structure = EventCommand {
		object_id: GameObjectId::new(std::u32::MAX, Owner::Root),
		field_id: 1050,
		event: vec![1, 2, 3, 4, 5],
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}

