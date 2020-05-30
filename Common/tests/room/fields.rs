use cheetah_relay_common::room::fields::GameObjectFields;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_game_object_fields() {
    let mut structure = GameObjectFields::default();
    structure.long_counters.insert(10, 100);
    structure.float_counters.insert(50, 100.0);
    structure.structures.insert(70, vec![1, 2, 3, 4, 5]);
    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}







