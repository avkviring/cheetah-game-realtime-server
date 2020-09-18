use cheetah_relay_common::network::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::network::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_c2s_meta_information() {
	let structure = C2SMetaCommandInformation {
		command_code: 100,
		timestamp: 1000,
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}

#[test]
fn test_codec_for_s2c_meta_information() {
	let structure = S2CMetaCommandInformation {
		command_code: 100,
		client: 1,
		timestamp: 1000,
	};
	should_decode_after_encode(&structure);
	should_encode_fail_when_buffer_is_not_enough(&structure);
	should_decode_fail_when_buffer_is_not_enough(&structure);
}
