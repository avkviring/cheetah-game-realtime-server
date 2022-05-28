use crate::proto::matches::relay::shared::{field_value::Variant, FieldValue};

impl From<i64> for FieldValue {
    fn from(value: i64) -> Self {
        FieldValue { variant: Some(Variant::Long(value))  }
    }
}

impl From<f64> for FieldValue {
    fn from(value: f64) -> Self {
        FieldValue { variant: Some(Variant::Double(value)) }
    }
}

impl From<Vec<u8>> for FieldValue {
    fn from(value: Vec<u8>) -> Self {
        FieldValue { variant: Some(Variant::Structure(value)) }
    }
}
