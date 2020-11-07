use std::ops::{BitAnd, Shl};

use serde::{Deserialize, Serialize};

use crate::constants::GroupType;

///
/// Группа доступа
///
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct AccessGroups(u64);

impl AccessGroups {
	pub fn contains_group(&self, group: u8) -> bool {
		let bits = (1 as u64).shl(group as u64);
		self.0.bitand(bits) == bits
	}
	
	pub fn contains_any(&self, groups: &AccessGroups) -> bool {
		self.0.bitand(groups.0) > 0
	}
	
	pub fn is_sub_groups(&self, groups: &AccessGroups) -> bool {
		groups.0.bitand(self.0) == self.0
	}
}


