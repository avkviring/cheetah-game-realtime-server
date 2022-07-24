use uuid::Uuid;
use ydb_steroids::converters::YDBValueConverter;

pub mod service;
pub mod storage;

pub struct Cookie(pub Uuid);

impl From<Uuid> for Cookie {
	fn from(uuid: Uuid) -> Self {
		Cookie(uuid)
	}
}

impl From<u128> for Cookie {
	fn from(uuid: u128) -> Self {
		Self(Uuid::from_u128(uuid))
	}
}

impl YDBValueConverter for Cookie {
	fn get_type_name(&self) -> &'static str {
		"String"
	}
	fn to_ydb_value(&self) -> ydb::Value {
		self.0.to_ydb_value()
	}
}
