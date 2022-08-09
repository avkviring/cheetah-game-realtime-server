use uuid::Uuid;

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
