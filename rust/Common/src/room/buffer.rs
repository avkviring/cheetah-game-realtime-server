use std::io::{Cursor, Error, ErrorKind, Read, Write};

use cheetah_game_realtime_protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use serde::{Deserialize, Serialize};

///
/// Бинарное значение поля
///
#[derive(Clone, PartialEq, Hash, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Buffer {
	#[serde(with = "serde_bytes")]
	pub buffer: Vec<u8>,
}

pub const MAX_BUFFER_SIZE: usize = 8192;

impl From<&[u8]> for Buffer {
	fn from(source: &[u8]) -> Self {
		Self { buffer: source.to_vec() }
	}
}

impl Buffer {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut result = Buffer::default();
		let size: usize = input.read_variable_u64()?.try_into().map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
		if size > MAX_BUFFER_SIZE {
			return Err(Error::new(ErrorKind::InvalidData, format!("Event buffer size to big {size}")));
		}

		let mut buffer = [0; MAX_BUFFER_SIZE];
		input.read_exact(&mut buffer[0..size])?;
		result.buffer = buffer[0..size].to_vec();
		Ok(result)
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.buffer.len() as u64)?;
		let res = out.write(&self.buffer.as_slice());
		match res {
			Ok(size) => {
				if size == self.buffer.len() as usize {
					Ok(())
				} else {
					Err(Error::new(ErrorKind::Interrupted, "not fully saved"))
				}
			}
			Err(e) => Err(e),
		}
	}
}
