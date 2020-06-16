use std::ops::{BitAnd, Shl};

use crate::constants::GroupType;
use crate::network::command::{Decoder, Encoder};
use crate::network::niobuffer::{NioBuffer, NioBufferError};

///
/// Группа доступа
///
#[derive(Debug, Clone, PartialEq)]
pub struct AccessGroups {
	pub groups: GroupType,
}

impl AccessGroups {
	pub fn contains_group(&self, group: u8) -> bool {
		let bits = (1 as u64).shl(group as u64);
		self.groups.bitand(bits) == bits
	}
	
	pub fn contains_any(&self, groups: &AccessGroups) -> bool {
		self.groups.bitand(groups.groups) > 0
	}
	
	pub fn is_sub_groups(&self, groups: &AccessGroups) -> bool {
		groups.groups.bitand(self.groups) == self.groups
	}
}

impl Default for AccessGroups {
	fn default() -> Self {
		AccessGroups::from(0)
	}
}

impl From<u64> for AccessGroups {
	fn from(groups: u64) -> AccessGroups {
		AccessGroups {
			groups
		}
	}
}

impl Decoder for AccessGroups {
	fn decode(buffer: &mut NioBuffer) -> Result<Self, NioBufferError> {
		match buffer.read_u64() {
			Ok(value) => {
				Result::Ok(AccessGroups::from(value))
			}
			Err(e) => {
				Result::Err(e)
			}
		}
	}
}

impl Encoder for AccessGroups {
	fn encode(&self, buffer: &mut NioBuffer) -> Result<(), NioBufferError> {
		buffer.write_u64(self.groups)
	}
}