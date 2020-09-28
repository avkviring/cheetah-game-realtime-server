use std::ops::{BitAnd, Shl};
use serde::{Deserialize, Serialize};
use crate::constants::GroupType;


///
/// Группа доступа
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
