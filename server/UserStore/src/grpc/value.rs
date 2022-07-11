use crate::grpc::userstore::{primitive_value, PrimitiveValue};

impl Into<PrimitiveValue> for i64 {
	fn into(self) -> PrimitiveValue {
		PrimitiveValue {
			pr: Some(primitive_value::Pr::Long(self)),
		}
	}
}

impl Into<PrimitiveValue> for f64 {
	fn into(self) -> PrimitiveValue {
		PrimitiveValue {
			pr: Some(primitive_value::Pr::Double(self)),
		}
	}
}

impl Into<PrimitiveValue> for String {
	fn into(self) -> PrimitiveValue {
		PrimitiveValue {
			pr: Some(primitive_value::Pr::String(self)),
		}
	}
}
