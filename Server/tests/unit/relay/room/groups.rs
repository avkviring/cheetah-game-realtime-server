use cheetah_relay_common::room::access::AccessGroups;

#[test]
fn create_group_from_vec() {
	let group = AccessGroups::from(0b1001);
	assert_eq!(group.contains_group(0), true);
	assert_eq!(group.contains_group(1), false);
	assert_eq!(group.contains_group(2), false);
	assert_eq!(group.contains_group(3), true);
}

#[test]
fn should_clone() {
	let group = AccessGroups::from(0b1001);
	assert_eq!(group.contains_group(0), true);
	assert_eq!(group.contains_group(1), false);
	assert_eq!(group.contains_group(2), false);
	assert_eq!(group.contains_group(3), true);
}

#[test]
fn contains_group_should_true_when_equals() {
	let group_a = AccessGroups::from(0b1001);
	let group_b = AccessGroups::from(0b1001);
	assert_eq!(group_a.contains_any(&group_b), true)
}

#[test]
fn contains_group_should_true_when_subgroup() {
	let group_a = AccessGroups::from(0b1001);
	let group_b = AccessGroups::from(0b1100);
	assert_eq!(group_a.contains_any(&group_b), true)
}

#[test]
fn contains_group_should_false() {
	let group_a = AccessGroups::from(0b1001);
	let group_b = AccessGroups::from(0b0110);
	assert_eq!(group_a.contains_any(&group_b), false)
}