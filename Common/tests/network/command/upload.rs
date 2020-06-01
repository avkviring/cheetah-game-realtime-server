use cheetah_relay_common::network::command::upload::{UploadGameObjectC2SCommand, UploadGameObjectS2CCommand};
use cheetah_relay_common::room::access::AccessGroups;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_upload_game_object_c2s_command() {
    let mut structure = UploadGameObjectC2SCommand {
        local_id: std::u32::MAX,
        access_groups: AccessGroups::from(std::u64::MAX),
        fields: Default::default(),
    };
    structure.fields.long_counters.insert(10, 100);
    structure.fields.float_counters.insert(20, 200.0);
    structure.fields.structures.insert(30, vec![1, 2, 3, 4, 5]);

    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}

#[test]
fn test_codec_for_upload_game_object_s2c_command() {
    let mut structure = UploadGameObjectS2CCommand {
        id: std::u64::MAX,
        fields: Default::default(),
    };
    structure.fields.long_counters.insert(10, 100);
    structure.fields.float_counters.insert(20, 200.0);
    structure.fields.structures.insert(30, vec![1, 2, 3, 4, 5]);

    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}