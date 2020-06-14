use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::Owner;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_unload_game_object_command() {
	let structure = UnloadGameObjectCommand {
		object_id: GameObjectId::new(std::u32::MAX, Owner::Root),
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}
