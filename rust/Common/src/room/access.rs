use std::ops::{BitAnd, Shl};

use serde::{Deserialize, Serialize};

///
/// Группа доступа
///
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Hash, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AccessGroups(pub u64);

impl AccessGroups {
	///
	/// Группа для включения всех групп
	///
	#[must_use]
	pub fn super_member_group() -> AccessGroups {
		AccessGroups(u64::MAX)
	}
}

impl AccessGroups {
	#[must_use]
	pub fn contains_group(&self, group: u8) -> bool {
		let bits = 1_u64.shl(u64::from(group));
		self.0.bitand(bits) == bits
	}

	#[must_use]
	pub fn contains_any(&self, groups: &AccessGroups) -> bool {
		self.0.bitand(groups.0) > 0
	}

	#[must_use]
	pub fn is_sub_groups(&self, groups: &AccessGroups) -> bool {
		groups.0.bitand(self.0) == self.0
	}
}

#[cfg(test)]
mod tests {
	use crate::room::access::AccessGroups;
	
	#[test]
	fn create_group_from_vec() {
		let group = AccessGroups(0b1001);
		assert!(group.contains_group(0));
		assert!(!group.contains_group(1));
		assert!(!group.contains_group(2));
		assert!(group.contains_group(3));
	}

	#[test]
	fn should_clone() {
		let group = AccessGroups(0b1001);
		assert!(group.contains_group(0));
		assert!(!group.contains_group(1));
		assert!(!group.contains_group(2));
		assert!(group.contains_group(3));
	}

	#[test]
	fn contains_group_should_true_when_equals() {
		let group_a = AccessGroups(0b1001);
		let group_b = AccessGroups(0b1001);
		assert!(group_a.contains_any(&group_b));
	}

	#[test]
	fn contains_group_should_true_when_subgroup() {
		let group_a = AccessGroups(0b1001);
		let group_b = AccessGroups(0b1100);
		assert!(group_a.contains_any(&group_b));
	}

	#[test]
	fn contains_group_should_false() {
		let group_a = AccessGroups(0b1001);
		let group_b = AccessGroups(0b0110);
		assert!(!group_a.contains_any(&group_b));
	}
}
