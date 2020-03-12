use crate::relay::room::groups::AccessGroups;

#[test]
fn create_group_from_vec() {
    let group = AccessGroups::from(&vec![25, 50]);
    assert_eq!(group.contains_group(0), false);
    assert_eq!(group.contains_group(25), true);
    assert_eq!(group.contains_group(50), true);
    assert_eq!(group.contains_group(55), false);
}

#[test]
fn should_clone() {
    let group = AccessGroups::from(&vec![25, 50]).clone();
    assert_eq!(group.contains_group(0), false);
    assert_eq!(group.contains_group(25), true);
    assert_eq!(group.contains_group(50), true);
    assert_eq!(group.contains_group(55), false);
}

#[test]
fn contains_group_should_true_when_equals() {
    let group_a = AccessGroups::from(&vec![25, 50]);
    let group_b = AccessGroups::from(&vec![25, 50]);
    assert_eq!(group_a.contains_groups(&group_b), true)
}

#[test]
fn contains_group_should_true_when_subgroup() {
    let group_a = AccessGroups::from(&vec![25, 50]);
    let group_b = AccessGroups::from(&vec![25]);
    assert_eq!(group_a.contains_groups(&group_b), true)
}

#[test]
fn contains_group_should_false() {
    let group_a = AccessGroups::from(&vec![25, 50]);
    let group_b = AccessGroups::from(&vec![15]);
    assert_eq!(group_a.contains_groups(&group_b), false)
}