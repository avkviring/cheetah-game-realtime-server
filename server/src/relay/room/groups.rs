use std::ops::{BitAnd, Shl, Shr};

/// Группа доступа
#[derive(Debug, Clone, PartialEq)]
pub struct AccessGroups {
	groups: u64,
}

#[derive(Debug)]
pub enum Access {
	READ,
	WRITE,
	ROOT,
}

impl AccessGroups {
	pub fn new() -> AccessGroups {
		AccessGroups::from(0)
	}
	
	pub fn contains_group(&self, group: u8) -> bool {
		let bits = (1 as u64).shl(group as u64);
		return self.groups.bitand(bits) == bits;
	}
	
	pub fn contains_any(&self, groups: &AccessGroups) -> bool {
		return self.groups.bitand(groups.groups) > 0;
	}
}

impl From<u64> for AccessGroups {
	fn from(groups: u64) -> AccessGroups {
		AccessGroups {
			groups
		}
	}
}
