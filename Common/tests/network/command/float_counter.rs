use std::u64::MAX;

use cheetah_relay_common::network::command::float_counter::{SetFloat64CounterCommand, IncrementFloat64CounterC2SCommand};

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};
use cheetah_relay_common::room::owner::ClientOwner;
use cheetah_relay_common::room::object::ClientGameObjectId;

#[test]
fn test_codec_for_set_float_counter_command() {
    let structure = SetFloat64CounterCommand {
        object_id: ClientGameObjectId::new(std::u32::MAX, ClientOwner::Root),
        field_id: 10500,
        value: 200.0,
    };
    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}

#[test]
fn test_codec_for_increment_float_counter_command() {
    let structure = IncrementFloat64CounterC2SCommand {
        object_id: ClientGameObjectId::new(std::u32::MAX, ClientOwner::Root),
        field_id: 10500,
        increment: 200.0,
    };
    should_decode_after_encode(&structure);
    should_encode_fail_when_buffer_is_not_enough(&structure);
    should_decode_fail_when_buffer_is_not_enough(&structure);
}

