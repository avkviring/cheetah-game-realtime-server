use crate::commands::FieldType;

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
