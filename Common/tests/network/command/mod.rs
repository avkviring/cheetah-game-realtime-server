use cheetah_relay_common::network::niobuffer::NioBuffer;

pub mod event;
pub mod unload;
pub mod structure;
pub mod long_counter;
pub mod float_counter;
pub mod upload;

pub fn create_buffer_with_capacity(size: usize) -> NioBuffer {
	let mut buffer = NioBuffer::new();
	buffer.set_limit(size).unwrap();
	buffer
}