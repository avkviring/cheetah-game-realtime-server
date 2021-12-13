use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

///
/// Запись/чтения целого числа с переменным количеством бит для хранения
///
pub trait VariableInt {
	fn write_variable_u64(&mut self, value: u64) -> std::io::Result<()>;
	fn write_variable_i64(&mut self, value: i64) -> std::io::Result<()>;

	fn read_variable_u64(&mut self) -> std::io::Result<u64>;
	fn read_variable_i64(&mut self) -> std::io::Result<i64>;
}

impl VariableInt for Cursor<&mut [u8]> {
	fn write_variable_u64(&mut self, value: u64) -> std::io::Result<()> {
		self.write_u64::<BigEndian>(value as u64)
	}

	fn write_variable_i64(&mut self, value: i64) -> std::io::Result<()> {
		self.write_i64::<BigEndian>(value)
	}

	fn read_variable_u64(&mut self) -> std::io::Result<u64> {
		self.read_u64::<BigEndian>()
	}

	fn read_variable_i64(&mut self) -> std::io::Result<i64> {
		self.read_i64::<BigEndian>()
	}
}
