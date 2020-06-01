use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_set_long_counter_command() {
    let structure = SetLongCounterCommand {
        global_object_id: std::u64::MAX,
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
        global_object_id: std::u64::MAX,
        field_id: std::u16::MAX,
        increment: 200,
    };
    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}