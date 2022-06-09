use std::time::Duration;

use uuid::Uuid;
use ydb::Value;

pub trait YDBValueConverter {
	fn get_type_name(&self) -> &'static str;
	fn to_ydb_value(&self) -> ydb::Value;
}

impl YDBValueConverter for &str {
	fn get_type_name(&self) -> &'static str {
		"Utf8"
	}

	fn to_ydb_value(&self) -> Value {
		Value::Utf8(self.to_string())
	}
}

impl YDBValueConverter for String {
	fn get_type_name(&self) -> &'static str {
		"Utf8"
	}

	fn to_ydb_value(&self) -> Value {
		Value::Utf8(self.clone())
	}
}

impl YDBValueConverter for Uuid {
	fn get_type_name(&self) -> &'static str {
		"String"
	}
	fn to_ydb_value(&self) -> Value {
		Value::String(ydb::Bytes::from(self.as_bytes().to_vec()))
	}
}

impl YDBValueConverter for Duration {
	fn get_type_name(&self) -> &'static str {
		"Timestamp"
	}
	fn to_ydb_value(&self) -> Value {
		ydb::Value::Timestamp(*self)
	}
}

impl YDBValueConverter for i32 {
	fn get_type_name(&self) -> &'static str {
		"Int32"
	}
	fn to_ydb_value(&self) -> Value {
		Value::Int32(*self)
	}
}

impl YDBValueConverter for Vec<u8> {
	fn get_type_name(&self) -> &'static str {
		"String"
	}
	fn to_ydb_value(&self) -> Value {
		Value::String(self.clone().into())
	}
}

impl<T> YDBValueConverter for &T
where
	T: YDBValueConverter,
{
	fn get_type_name(&self) -> &'static str {
		(*self).get_type_name()
	}
	fn to_ydb_value(&self) -> Value {
		(*self).to_ydb_value()
	}
}
