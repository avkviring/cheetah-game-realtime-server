use std::io::Cursor;
use std::io::ErrorKind;

use byteorder::{ReadBytesExt, WriteBytesExt};

///
/// Тип данных поля
///
#[repr(C)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum FieldType {
	Long = 1,
	Double = 2,
	Structure = 3,
	Event = 4,
}

impl hash32::Hash for FieldType {
	fn hash<H>(&self, state: &mut H)
	where
		H: hash32::Hasher,
	{
		(*self as u8).hash(state);
	}
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
		let code: u8 = *self as u8;
		out.write_u8(code)
	}

	pub fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let value = input.read_u8()?;
		Ok(match value {
			1 => FieldType::Long,
			2 => FieldType::Double,
			3 => FieldType::Structure,
			4 => FieldType::Event,
			_ => return Err(std::io::Error::new(ErrorKind::InvalidData, format!("{}", value))),
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

impl ToFieldType for Vec<u8> {
	fn to_field_type() -> FieldType {
		FieldType::Structure
	}
}

impl ToFieldType for &[u8] {
	fn to_field_type() -> FieldType {
		FieldType::Structure
	}
}

pub type FieldId = u16;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Field {
	pub id: FieldId,
	pub field_type: FieldType,
}
