use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ydb_steroids::converters::YDBValueConverter;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub struct User(pub(crate) Uuid);

impl Default for User {
	fn default() -> Self {
		Self(Uuid::new_v4())
	}
}

impl From<Uuid> for User {
	fn from(uuid: Uuid) -> Self {
		User(uuid)
	}
}

impl From<ydb::Bytes> for User {
	fn from(value: ydb::Bytes) -> Self {
		let vec: Vec<_> = value.try_into().unwrap();
		User(Uuid::from_slice(vec.as_slice()).unwrap())
	}
}

impl From<u128> for User {
	fn from(value: u128) -> Self {
		Uuid::from_u128(value).into()
	}
}

impl YDBValueConverter for User {
	fn get_type_name(&self) -> &'static str {
		"String"
	}
	fn to_ydb_value(&self) -> ydb::Value {
		self.0.to_ydb_value()
	}
}
