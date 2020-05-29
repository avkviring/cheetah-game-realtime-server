use cheetah_relay_common::network::niobuffer::NioBuffer;

#[test]
fn should_write_and_read() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(10).ok().unwrap();
	buffer.write_u16(100).ok().unwrap();
	buffer.write_u32(1000).ok().unwrap();
	buffer.write_u64(10000).ok().unwrap();
	
	buffer.flip();
	assert_eq!(buffer.read_u8().ok().unwrap(), 10);
	assert_eq!(buffer.read_u16().ok().unwrap(), 100);
	assert_eq!(buffer.read_u32().ok().unwrap(), 1000);
	assert_eq!(buffer.read_u64().ok().unwrap(), 10000);
}

#[test]
fn should_none_when_cannot_read() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	assert_eq!(buffer.read_u8().is_err(), true)
}

#[test]
fn should_none_when_cannot_write() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	assert_eq!(buffer.write_u8(10).is_err(), true)
}

#[test]
fn should_compact() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(10).ok().unwrap();
	buffer.write_u16(100).ok().unwrap();
	buffer.write_u32(1000).ok().unwrap();
	buffer.write_u64(10000).ok().unwrap();
	
	buffer.flip();
	
	buffer.read_u8().ok().unwrap();
	buffer.read_u16().ok().unwrap();
	
	buffer.compact();
	buffer.write_u64(12345).ok().unwrap();
	buffer.flip();
	
	assert_eq!(buffer.read_u32().ok().unwrap(), 1000);
	assert_eq!(buffer.read_u64().ok().unwrap(), 10000);
	assert_eq!(buffer.read_u64().ok().unwrap(), 12345);
}

#[test]
fn should_remaining() {
	let mut buffer = NioBuffer::new();
	assert_eq!(buffer.has_remaining(), true);
	
	buffer.write_u8(10).ok().unwrap();
	buffer.flip();
	assert_eq!(buffer.has_remaining(), true);
	assert_eq!(buffer.remaining(), 1);
	buffer.read_u8().ok().unwrap();
	assert_eq!(buffer.has_remaining(), false);
	assert_eq!(buffer.remaining(), 0);
	
	buffer.compact();
	assert_eq!(buffer.has_remaining(), true);
	assert_eq!(buffer.remaining(), NioBuffer::NIO_BUFFER_CAPACITY);
}

#[test]
fn should_mark() {
	let mut buffer = NioBuffer::new();
	buffer.write_u8(10).ok().unwrap();
	buffer.write_u16(100).ok().unwrap();
	buffer.write_u32(1000).ok().unwrap();
	
	buffer.flip();
	
	buffer.read_u8().ok().unwrap();
	buffer.mark();
	buffer.read_u16().ok().unwrap();
	buffer.read_u32().ok().unwrap();
	buffer.reset().ok().unwrap();
	
	assert_eq!(buffer.read_u16().ok().unwrap(), 100);
	assert_eq!(buffer.read_u32().ok().unwrap(), 1000);
}

#[test]
fn should_error_when_mark_not_found() {
	let mut buffer = NioBuffer::new();
	assert_eq!(buffer.reset().is_err(), true);
	buffer.mark();
	assert_eq!(buffer.reset().is_ok(), true);
	assert_eq!(buffer.reset().is_err(), true);
}


#[test]
fn should_write_bytes() {
	let mut buffer = NioBuffer::new();
	let bytes = vec![1, 2, 3];
	buffer.write_bytes(bytes.as_slice()).ok().unwrap();
	buffer.flip();
	assert_eq!(buffer.read_u8().ok().unwrap(), 1);
	assert_eq!(buffer.read_u8().ok().unwrap(), 2);
	assert_eq!(buffer.read_u8().ok().unwrap(), 3);
}

#[test]
fn should_to_slice() {
	let mut buffer = NioBuffer::new();
	let bytes = vec![1, 2, 3];
	buffer.write_bytes(bytes.as_slice()).ok().unwrap();
	buffer.flip();
	buffer.read_u8().unwrap();
	let slice: &[u8] = buffer.to_slice();
	assert_eq!(slice, vec![2, 3].as_slice())
}

#[test]
fn should_change_position() {
	let mut buffer = NioBuffer::new();
	let bytes = vec![1, 2, 3];
	buffer.write_bytes(bytes.as_slice()).ok().unwrap();
	buffer.flip();
	assert_eq!(buffer.position(), 0);
	buffer.set_position(buffer.position() + 1).unwrap();
	assert_eq!(buffer.position(), 1);
	assert_eq!(buffer.read_u8().unwrap(), 2);
}

#[test]
fn should_fail_when_change_position() {
	let mut buffer = NioBuffer::new();
	buffer.flip();
	assert_eq!(buffer.set_position(1).is_err(), true);
}

#[test]
fn should_set_limit() {
	let mut buffer = NioBuffer::new();
	assert_eq!(buffer.set_limit(10).is_ok(), true);
}

#[test]
fn should_error_when_set_limit_overflow() {
	let mut buffer = NioBuffer::new();
	buffer.set_position(100).unwrap();
	assert_eq!(buffer.set_limit(10).is_err(), true);
}