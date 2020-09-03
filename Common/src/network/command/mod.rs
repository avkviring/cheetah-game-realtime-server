use crate::network::niobuffer::{NioBuffer, NioBufferError};

pub mod event;
pub mod unload;
pub mod float_counter;
pub mod long_counter;
pub mod structure;
pub mod load;


pub trait Encoder {
	///
	/// Преобразовать команду в поток байт
	///
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError>;
}

pub trait Decoder where Self: Sized {
	///
	/// Преобразовать поток байт в команду
	///
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError>;
}

pub trait CommandCode {
	const COMMAND_CODE: u8;
}