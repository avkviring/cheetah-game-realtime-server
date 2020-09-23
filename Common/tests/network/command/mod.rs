use cheetah_relay_common::network::command::{Decoder, Encoder};
use cheetah_relay_common::network::niobuffer::{NioBuffer, NioBufferError};
use std::fmt::Debug;

pub mod event;
pub mod unload;
pub mod structure;
pub mod long_counter;
pub mod float_counter;
pub mod load;
pub mod meta;

pub fn create_buffer_with_capacity(size: usize) -> NioBuffer {
    let mut buffer = NioBuffer::new();
    buffer.set_limit(size).unwrap();
    buffer
}

pub fn should_decode_after_encode<T: Encoder + Decoder + PartialEq + Debug>(structure: &T) {
    let mut buffer = &mut NioBuffer::new();
    structure.encode(buffer).unwrap();
    buffer.flip();
    let decoded = (Decoder::decode(&mut buffer) as Result<T, NioBufferError>).unwrap();
    assert_eq!(&decoded, structure);
}

///
/// проверяем кодирование структуры в буфер
/// размер которого меньше чем необходимо для кодирования
///
pub fn should_encode_fail_when_buffer_is_not_enough<T: Encoder>(structure: &T) {
    let mut buffer = NioBuffer::new();
    structure.encode(&mut buffer).unwrap();
    buffer.flip();
    let size = buffer.remaining();

    for i in 0..size {
        let mut buffer = create_buffer_with_capacity(i);
        assert_eq!(structure.encode(&mut buffer).is_ok(), false, "{}", i)
    }
}


pub fn should_decode_fail_when_buffer_is_not_enough<T: Encoder + Decoder>(structure: &T) {
    let mut buffer = NioBuffer::new();
    structure.encode(&mut buffer).unwrap();
    buffer.flip();
    let size = buffer.remaining();

    for i in 0..size {
        buffer.set_position(0).unwrap();
        buffer.set_limit(i).unwrap();
        assert_eq!((Decoder::decode(&mut buffer) as Result<T, NioBufferError>).is_ok(), false)
    }
}
