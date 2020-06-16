use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_set_structure_command() {
	let structure = StructureCommand {
		object_id: ClientGameObjectId::new(std::u32::MAX, ClientOwner::Root),
		field_id: std::u16::MAX,
		structure: vec![1, 2, 3, 4, 5],
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}
