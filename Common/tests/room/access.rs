use cheetah_relay_common::network::command::{Decoder, Encoder};
use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::room::access::AccessGroups;

use crate::network::command::create_buffer_with_capacity;

#[test]
fn should_encode_decode() {
	let access = AccessGroups::from(123);
	let buffer = &mut NioBuffer::new();
	access.encode(buffer);
	buffer.flip();
	assert_eq!(AccessGroups::decode(buffer).unwrap(), access);
}


#[test]
fn should_dont_encode_when_buffer_is_full() {
	let access = AccessGroups::from(123);
	let buffer = &mut create_buffer_with_capacity(0);
	assert_eq!(access.encode(buffer).is_ok(), false);
}

#[test]
fn should_dont_encode_when_data_is_not_enough() {
	let buffer = &mut create_buffer_with_capacity(0);
	assert_eq!(AccessGroups::decode(buffer).is_ok(), false);
}