use std::io::Cursor;
use std::io::ErrorKind;

use byteorder::{ReadBytesExt, WriteBytesExt};

use serde::{Deserialize, Serialize};

use crate::room::buffer::Buffer;

///
/// Тип данных поля
///
#[repr(C)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FieldType {
	Long,
	Double,
	Structure,
	Event,
}

impl ToString for FieldType {
	fn to_string(&self) -> String {
		match self {
			FieldType::Long => "long",
			FieldType::Double => "double",
			FieldType::Structure => "structure",
			FieldType::Event => "event",
		}
		.into()
	}
}

impl FieldType {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		let code = match self {
			FieldType::Long => 1,
			FieldType::Double => 2,
			FieldType::Structure => 3,
			FieldType::Event => 4,
		};
		out.write_u8(code)
	}

	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = input.read_u8()?;
		Ok(match value {
			1 => FieldType::Long,
			2 => FieldType::Double,
			3 => FieldType::Structure,
			4 => FieldType::Event,
			_ => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("Read FieldType with code {value}"))),
		})
	}
}

pub trait ToFieldType {
	fn to_field_type() -> FieldType;
}

impl ToFieldType for i64 {
	fn to_field_type() -> FieldType {
		FieldType::Long
	}
}

impl ToFieldType for f64 {
	fn to_field_type() -> FieldType {
		FieldType::Double
	}
}

impl ToFieldType for Buffer {
	fn to_field_type() -> FieldType {
		FieldType::Structure
	}
}

pub type FieldId = u16;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Field {
	pub id: FieldId,
	pub field_type: FieldType,
}

#[cfg(test)]
mod test {
	use std::io::Cursor;

	use crate::room::field::FieldType;

	#[test]
	fn test() {
		check(FieldType::Long);
		check(FieldType::Structure);
		check(FieldType::Double);
		check(FieldType::Event);
	}

	fn check(original: FieldType) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		original.encode(&mut cursor).unwrap();

		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let result = FieldType::decode(&mut read_cursor).unwrap();

		assert_eq!(original, result);
	}
}
