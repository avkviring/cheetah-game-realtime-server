use std::ops::{BitAnd, Shl, Shr};

/// Группа доступа
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
		AccessGroups::from(Vec::<u8>::new())
	}
	
	pub fn contains_group(&self, group: u8) -> bool {
		let bits = (1 as u64).shl(group as u64);
		return self.groups.bitand(bits) == bits;
	}
	
	pub fn contains_any(&self, groups: &AccessGroups) -> bool {
		return self.groups.bitand(groups.groups) > 0;
	}
}


impl Clone for AccessGroups {
	fn clone(&self) -> Self {
		AccessGroups {
			groups: self.groups.clone()
		}
	}
}


impl From<Vec<u8>> for AccessGroups {
	fn from(groups: Vec<u8>) -> AccessGroups {
		let mut value: u64 = 0;
		for group in groups {
			let bit = (1 as u64).shl(group as u64);
			value += bit
		}
		AccessGroups {
			groups: value
		}
	}
}
