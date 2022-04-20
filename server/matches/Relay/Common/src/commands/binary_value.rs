use std::io::{Cursor, Error, ErrorKind, Read, Write};

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};

///
/// Бинарное значение поля
///
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BinaryValue(heapless::Vec<u8, 256>);

impl From<&[u8]> for BinaryValue {
	fn from(source: &[u8]) -> Self {
		BinaryValue(heapless::Vec::<u8, 256>::from_slice(source).unwrap())
	}
}
impl BinaryValue {
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut result = BinaryValue { 0: Default::default() };
		let size = input.read_variable_u64()? as usize;
		if size > result.0.capacity() {
			return Err(Error::new(
				ErrorKind::InvalidData,
				format!("Event buffer size to big {}", size),
			));
		}
		unsafe {
			result.0.set_len(size);
		}
		input.read_exact(&mut result.0[0..size])?;
		Ok(result)
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.0.len() as u64)?;
		out.write_all(self.0.as_slice())
	}

	pub(crate) fn len(&self) -> usize {
		self.0.len()
	}
}
