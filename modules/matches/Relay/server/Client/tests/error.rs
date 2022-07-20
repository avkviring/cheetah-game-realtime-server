use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::BufferFFI;

#[test]
pub fn should_last_error() {
	let code = ffi::client::destroy_client(100);
	assert_eq!(code, 2);
	let mut buffer = BufferFFI::default();
	ffi::client::get_last_error_msg(&mut buffer);
	let message = String::from_utf8_lossy(&buffer.buffer[0..buffer.len as usize]);
	assert_eq!(message.to_string(), "ClientNotFound(100)");
}
