use std::fmt;
use std::io::{Cursor, Error, ErrorKind, Read, Write};

use cheetah_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};

///
/// Бинарное значение поля
///
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Hash, Eq)]
pub struct Buffer {
	pub len: usize,
	pub pos: usize,
	// используется в C#
	pub buffer: [u8; NIO_BUFFER_MAX_SIZE],
}

impl fmt::Debug for Buffer {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_list().entries(self.buffer[0..self.len as usize].iter()).finish()
	}
}

impl Buffer {
	pub fn as_slice(&self) -> &[u8] {
		&self.buffer[0..self.len as usize]
	}
}

impl Default for Buffer {
	fn default() -> Self {
		Self {
			len: 0,
			pos: 0,
			buffer: [0; NIO_BUFFER_MAX_SIZE],
		}
	}
}

pub const NIO_BUFFER_MAX_SIZE: usize = 255;

impl From<&[u8]> for Buffer {
	fn from(source: &[u8]) -> Self {
		let mut result = Self {
			len: source.len(),
			pos: 0,
			buffer: [0; NIO_BUFFER_MAX_SIZE],
		};

		result.buffer[0..source.len()].copy_from_slice(source);
		result
	}
}

impl Buffer {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut result = Buffer::default();
		let size: usize = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
		if size > NIO_BUFFER_MAX_SIZE {
			return Err(Error::new(ErrorKind::InvalidData, format!("Event buffer size to big {size}")));
		}
		result.len = size;
		result.pos = 0;
		input.read_exact(&mut result.buffer[0..size])?;
		Ok(result)
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.len as u64)?;
		let res = out.write(&self.buffer[0..self.len as usize]);
		match res {
			Ok(size) => {
				if size == self.len as usize {
					Ok(())
				} else {
					Err(Error::new(ErrorKind::Interrupted, "not fully saved"))
				}
			}
			Err(e) => Err(e),
		}
	}
}