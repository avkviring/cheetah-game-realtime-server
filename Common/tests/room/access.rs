use cheetah_relay_common::room::access::AccessGroups;

use crate::network::command::{should_decode_after_encode, should_decode_fail_when_buffer_is_not_enough, should_encode_fail_when_buffer_is_not_enough};

#[test]
fn test_codec_for_access_groups() {
	let access_groups = AccessGroups::from(std::u64::MAX);
	should_decode_after_encode(&access_groups);
	should_encode_fail_when_buffer_is_not_enough(&access_groups);
	should_decode_fail_when_buffer_is_not_enough(&access_groups);
}

#[test]
fn should_has_subgroup() {
	let access_groups = AccessGroups::from(0b111);
	assert!(AccessGroups::from(0b100).is_sub_groups(&access_groups));
	assert!(AccessGroups::from(0b111).is_sub_groups(&access_groups));
	assert!(!AccessGroups::from(0b1111).is_sub_groups(&access_groups));
}