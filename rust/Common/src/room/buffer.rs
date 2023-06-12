use std::fmt;
use std::io::{Cursor, Error, ErrorKind, Read, Write};

use cheetah_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use cheetah_protocol::frame::packets_collector::PACKET_SIZE;

///
/// Бинарное значение поля
///
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Hash, Eq)]
pub struct Buffer {
	pub len: u16,
	// используется в C#
	pub pos: u16,
	pub buffer: [u8; BUFFER_SIZE],
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
			buffer: [0; BUFFER_SIZE],
		}
	}
}

pub const BUFFER_SIZE: usize = 8192;

impl From<&[u8]> for Buffer {
	fn from(source: &[u8]) -> Self {
		let mut result = Self {
			len: source.len() as u16,
			pos: 0,
			buffer: [0; BUFFER_SIZE],
		};

		result.buffer[0..source.len()].copy_from_slice(source);
		result
	}
}

impl Buffer {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut result = Buffer::default();
		let size: usize = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
		if size > BUFFER_SIZE {
			return Err(Error::new(ErrorKind::InvalidData, format!("Event buffer size to big {size}")));
		}
		result.len = size as u16;
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
