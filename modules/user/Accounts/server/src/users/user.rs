use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl From<u128> for User {
	fn from(value: u128) -> Self {
		Uuid::from_u128(value).into()
	}
}
