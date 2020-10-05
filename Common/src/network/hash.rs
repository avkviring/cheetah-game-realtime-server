use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::fmt;

use serde::{Deserialize, Serialize};

pub type UserPrivateKey = [u8; 32];
pub type UserPublicKey = [u8; 4];

/// Hash клиента и комнаты
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HashValue {
	pub value: [u8; HashValue::SIZE]
}


impl Display for HashValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "[0x").ok().unwrap();
		for b in self.value.iter() {
			write!(f, "{:X}", b).ok().unwrap();
		}
		write!(f, "]").ok().unwrap();
		fmt::Result::Ok(())
	}
}

impl From<&HashValue> for String {
	fn from(hash: &HashValue) -> Self {
		String::from_utf8(hash.value.to_vec()).unwrap()
	}
}

impl From<&str> for HashValue {
	fn from(str: &str) -> Self {
		let mut bytes = [b'_'; HashValue::SIZE];
		let src_bytes = str.as_bytes();
		let size = min(bytes.len(), src_bytes.len());
		bytes[..size].copy_from_slice(&src_bytes[..size]);
		
		
		HashValue {
			value: bytes
		}
	}
}

impl From<&[u8]> for HashValue {
	fn from(value: &[u8]) -> Self {
		let mut data = [0; HashValue::SIZE];
		data.copy_from_slice(value);
		HashValue {
			value: data
		}
	}
}

impl HashValue {
	pub const SIZE: usize = 16;
}
