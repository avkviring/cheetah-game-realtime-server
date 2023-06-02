use std::io::{Cursor, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

///
/// Запись/чтения целого числа с переменным количеством бит для хранения
///
pub trait VariableIntWriter {
	fn write_variable_u64(&mut self, value: u64) -> std::io::Result<()>;
	fn write_variable_i64(&mut self, value: i64) -> std::io::Result<()>;
}

pub trait VariableIntReader {
	fn read_variable_u64(&mut self) -> std::io::Result<u64>;
	fn read_variable_i64(&mut self) -> std::io::Result<i64>;
}
const U8_MAX: u64 = 249;
const U9_MARKER: u8 = 250;
const U16_MARKER: u8 = 251;
const U24_MARKER: u8 = 252;
const U32_MARKER: u8 = 253;
const U48_MARKER: u8 = 254;
const U64_MARKER: u8 = 255;

impl VariableIntWriter for Cursor<&mut [u8]> {
	#[allow(clippy::cast_possible_truncation)]
	fn write_variable_u64(&mut self, value: u64) -> std::io::Result<()> {
		if value < U8_MAX {
			return self.write_u8(value as u8);
		};

		if value < U8_MAX + 255 {
			self.write_u8(U9_MARKER)?;
			return self.write_u8((value - U8_MAX) as u8);
		};

		if value < u64::from(u16::MAX) {
			self.write_u8(U16_MARKER)?;
			return self.write_u16::<BigEndian>(value as u16);
		};

		if value < u64::from(u16::MAX) * u64::from(u8::MAX) {
			self.write_u8(U24_MARKER)?;
			return self.write_u24::<BigEndian>(value as u32);
		}

		if value < u64::from(u32::MAX) {
			self.write_u8(U32_MARKER)?;
			return self.write_u32::<BigEndian>(value as u32);
		};

		if value < u64::from(u32::MAX) * u64::from(u8::MAX) * u64::from(u8::MAX) {
			self.write_u8(U48_MARKER)?;
			return self.write_u48::<BigEndian>(value);
		};

		self.write_u8(U64_MARKER)?;
		self.write_u64::<BigEndian>(value)
	}

	#[allow(clippy::cast_possible_wrap)]
	#[allow(clippy::cast_sign_loss)]
	fn write_variable_i64(&mut self, value: i64) -> std::io::Result<()> {
		let zigzag = if value < 0 { !(value as u64) * 2 + 1 } else { (value as u64) * 2 };
		self.write_variable_u64(zigzag)
	}
}
impl VariableIntReader for Cursor<&[u8]> {
	#[allow(clippy::cast_possible_truncation)]
	fn read_variable_u64(&mut self) -> std::io::Result<u64> {
		let first = self.read_u8()?;
		if first < U8_MAX as u8 {
			return Ok(u64::from(first));
		};
		Ok(match first {
			U9_MARKER => U8_MAX + u64::from(self.read_u8()?),
			U16_MARKER => u64::from(self.read_u16::<BigEndian>()?),
			U24_MARKER => u64::from(self.read_u24::<BigEndian>()?),
			U32_MARKER => u64::from(self.read_u32::<BigEndian>()?),
			U48_MARKER => self.read_u48::<BigEndian>()?,
			U64_MARKER => self.read_u64::<BigEndian>()?,
			_ => {
				return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Variable int marker not valid {first}")));
			}
		})
	}

	#[allow(clippy::cast_possible_wrap)]
	fn read_variable_i64(&mut self) -> std::io::Result<i64> {
		let unsigned = self.read_variable_u64()?;
		Ok(if unsigned % 2 == 0 { unsigned / 2 } else { !(unsigned / 2) } as i64)
	}
}

#[cfg(test)]
mod test {
	use std::io::Cursor;

	use crate::codec::variable_int::{VariableIntReader, VariableIntWriter, U8_MAX, U9_MARKER};

	#[test]
	fn test_u64() {
		check_u64(U8_MAX - 1, 1);
		check_u64(U8_MAX, 2);
		check_u64(U8_MAX + 255 - 1, 2);
		check_u64(u64::from(u16::MAX - 1), 3);
		check_u64(u64::from(u16::MAX) * u64::from(u8::MAX) - 1, 4);
		check_u64(u64::from(u32::MAX - 1), 5);
		check_u64(u64::from(u32::MAX) * u64::from(u8::MAX) - 1, 7);
		check_u64(u64::MAX - 1, 9);
	}

	#[test]
	fn test_i64() {
		check_i64(-1, 1);
		check_i64(1, 1);
		check_i64((i64::from(U9_MARKER) + 255 - 2) / 2, 2);
		check_i64(-(i64::from(U9_MARKER) + 255 - 2) / 2, 2);
	}

	fn check_u64(value: u64, size: u64) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		cursor.write_variable_u64(value).unwrap();
		assert_eq!(cursor.position(), size);
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		assert_eq!(read_cursor.read_variable_u64().unwrap(), value);
		assert_eq!(write_position, read_cursor.position());
	}

	fn check_i64(value: i64, size: u64) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		cursor.write_variable_i64(value).unwrap();
		assert_eq!(cursor.position(), size);
		let write_position = cursor.position();

		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		assert_eq!(read_cursor.read_variable_i64().unwrap(), value);
		assert_eq!(write_position, read_cursor.position());
	}
}
