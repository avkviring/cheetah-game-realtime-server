use bit_array::BitArray;
use typenum::{U64, Unsigned};

/// Группа доступа
pub struct AccessGroups {
    groups: BitArray<u64, U64>,
}

impl AccessGroups {

    pub fn new() -> AccessGroups {
        AccessGroups::new_from_vec(&Vec::<u8>::new())
    }

    pub fn new_from_groups(groups: &AccessGroups) -> AccessGroups {
        AccessGroups {
            groups: groups.groups.clone()
        }
    }

    pub fn new_from_vec(groups: &Vec<u8>) -> AccessGroups {
        let mut bit_array = BitArray::<u64, U64>::from_elem(false);
        for x in groups {
            bit_array.set(*x as usize, true)
        }
        AccessGroups {
            groups: bit_array
        }
    }

    pub fn contains_group(&self, group: usize) -> bool {
        return self.groups.get(group).unwrap();
    }

    pub fn contains_groups(&self, groups: &AccessGroups) -> bool {
        for i in 0..groups.groups.len() {
            if groups.contains_group(i) {
                if !self.contains_group(i) {
                    return false;
                }
            }
        }
        return true;
    }
}