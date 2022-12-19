use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::{
	commands::FieldType,
	protocol::codec::variable_int::{VariableIntReader, VariableIntWriter},
};

use super::{binary_value::BinaryValue, field::ToFieldType};

#[derive(Debug, Clone, PartialEq, Copy)]
#[repr(u8)]
pub enum FieldValue {
	Long(i64),
	Double(f64),
	Structure(BinaryValue),
}

impl FieldValue {
	#[must_use]
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
			FieldValue::Structure(v) => v.encode(out),
		}
	}

	pub fn decode<T: Into<Self> + ToFieldType>(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let field_type = T::to_field_type();
		Ok(match field_type {
			FieldType::Long => input.read_variable_i64()?.into(),
			FieldType::Double => input.read_f64::<BigEndian>()?.into(),
			FieldType::Structure => BinaryValue::decode(input)?.into(),
			FieldType::Event => panic!("Event type is not supported"),
		})
	}
}

impl From<FieldValue> for f64 {
	fn from(from: FieldValue) -> Self {
		if let FieldValue::Double(v) = from {
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

impl From<BinaryValue> for FieldValue {
	fn from(value: BinaryValue) -> Self {
		FieldValue::Structure(value)
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

impl AsRef<BinaryValue> for FieldValue {
	fn as_ref(&self) -> &BinaryValue {
		if let FieldValue::Structure(v) = self {
			v
		} else {
			panic!("FieldValue had unexpected variant, expected FieldValue::Structure")
		}
	}
}

#[cfg(test)]
mod test {
	use std::io::Cursor;

	use crate::commands::binary_value::BinaryValue;
	use crate::commands::field::ToFieldType;
	use crate::commands::FieldValue;

	#[test]
	fn test() {
		check::<i64>(FieldValue::Long(100));
		check::<f64>(FieldValue::Double(100.100));
		check::<BinaryValue>(FieldValue::Structure(BinaryValue::from([1, 2, 3].as_ref())));
	}

	fn check<T: Into<FieldValue> + ToFieldType>(original: FieldValue) {
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		original.encode(&mut cursor).unwrap();

		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let result = FieldValue::decode::<T>(&mut read_cursor).unwrap();

		assert_eq!(original, result);
	}
}
