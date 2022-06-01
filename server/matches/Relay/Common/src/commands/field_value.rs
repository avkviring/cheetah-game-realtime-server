use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
	commands::FieldType,
	protocol::codec::variable_int::{VariableIntReader, VariableIntWriter},
};

use super::{binary_value::BinaryValue, field_type::ToFieldType};

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
	Long(i64),
	Double(f64),
	Structure(Vec<u8>),
}

impl FieldValue {
	pub fn field_type(&self) -> FieldType {
		match self {
			FieldValue::Long(_) => FieldType::Long,
			FieldValue::Double(_) => FieldType::Double,
			FieldValue::Structure(_) => FieldType::Structure,
		}
	}
}

impl FieldValue {
	pub fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		match self {
			FieldValue::Long(v) => out.write_variable_i64(*v),
			FieldValue::Double(v) => out.write_f64::<BigEndian>(*v),
			FieldValue::Structure(v) => {
				let bv: BinaryValue = v.as_slice().into();
				bv.encode(out)
			}
		}
	}

	pub fn decode<T: Into<Self> + ToFieldType>(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let field_type = T::to_field_type();
		Ok(match field_type {
			FieldType::Long => input.read_variable_i64()?.into(),
			FieldType::Double => input.read_f64::<BigEndian>()?.into(),
			FieldType::Structure => BinaryValue::decode(input)?.as_slice().into(),
			FieldType::Event => panic!("Event type is not supported"),
		})
	}
}

impl Into<f64> for FieldValue {
	fn into(self) -> f64 {
		if let FieldValue::Double(v) = self {
			v
		} else {
			panic!("Type mismatch: FieldValue contained wrong type");
		}
	}
}

impl From<i64> for FieldValue {
	fn from(value: i64) -> Self {
		FieldValue::Long(value)
	}
}

impl From<f64> for FieldValue {
	fn from(value: f64) -> Self {
		FieldValue::Double(value)
	}
}

impl From<&[u8]> for FieldValue {
	fn from(value: &[u8]) -> Self {
		FieldValue::Structure(value.into())
	}
}

impl From<Vec<u8>> for FieldValue {
	fn from(vec: Vec<u8>) -> Self {
		vec.as_slice().into()
	}
}

impl AsRef<f64> for FieldValue {
	fn as_ref(&self) -> &f64 {
		if let FieldValue::Double(v) = self {
			v
		} else {
			panic!("FieldValue had unexpected variant, expected FieldValue::Double")
		}
	}
}

impl AsRef<i64> for FieldValue {
	fn as_ref(&self) -> &i64 {
		if let FieldValue::Long(v) = self {
			v
		} else {
			panic!("FieldValue had unexpected variant, expected FieldValue::Long")
		}
	}
}

impl AsRef<Vec<u8>> for FieldValue {
	fn as_ref(&self) -> &Vec<u8> {
		if let FieldValue::Structure(v) = self {
			v
		} else {
			panic!("FieldValue had unexpected variant, expected FieldValue::Structure")
		}
	}
}
