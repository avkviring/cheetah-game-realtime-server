use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::fmt;

/// Hash клиента и комнаты
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashValue {
	pub value: [u8; HashValue::SIZE]
}

pub trait ToHashValue {
	fn to_hash_value(&self) -> HashValue;
}

impl ToHashValue for str {
	fn to_hash_value(&self) -> HashValue {
		HashValue::from(self)
	}
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

impl From<&str> for HashValue {
	fn from(str: &str) -> Self {
		let mut bytes = [0 as u8; HashValue::SIZE];
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
