use super::{Error, Rule};
use crate::proto::matches::relay::types as relay;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type GroupAlias = String;

#[derive(Default)]
pub struct GroupResolver {
    groups: HashMap<GroupAlias, (u32, u64)>,
}

impl GroupResolver {
    pub fn build(
        groups: HashMap<PathBuf, HashMap<GroupAlias, u64>>,
    ) -> (HashMap<PathBuf, Self>, Self) {
        let mut last_id = 1_u32;
        let mut global = HashMap::new();
        let local: HashMap<_, _> = groups
            .into_iter()
            .map(|(path, raw)| {
                let mut groups = HashMap::default();
                for (name, mask) in raw {
                    let id = last_id;
                    last_id += 1;
                    groups.insert(name, (id, mask));
                }
                global.extend(groups.clone());
                (path, GroupResolver { groups })
            })
            .collect();

        (local, GroupResolver { groups: global })
    }

    pub fn resolve_mask(&self, group: &str) -> Option<u64> {
        self.groups.get(group).map(|(_, mask)| *mask)
    }

    pub fn resolve(
        &self,
        group: &str,
        rule: Rule,
        path: &Path,
    ) -> Result<relay::GroupsPermissionRule, Error> {
        let permission = match rule {
            Rule::Deny => relay::PermissionLevel::Deny as i32,
            Rule::ReadOnly => relay::PermissionLevel::Ro as i32,
            Rule::ReadWrite => relay::PermissionLevel::Rw as i32,
        };

        self.groups
            .get(group)
            .copied()
            .map(|(_, groups)| relay::GroupsPermissionRule { groups, permission })
            .ok_or_else(|| Error::GroupNotFound(path.into(), group.into()))
    }
}

#[cfg(test)]
#[test]
fn resolver() {
    struct Test<'a> {
        path: PathBuf,
        group: &'a str,
        mask: u64,
    }

    let a = Test {
        path: "/dir/groups".into(),
        group: "test",
        mask: 12345,
    };
    let b = Test {
        path: "/dir/xxxx".into(),
        group: "test2",
        mask: 4321,
    };

    let mut groups = HashMap::new();
    groups.insert(a.path.clone(), {
        let mut file = HashMap::default();
        file.insert(a.group.into(), a.mask);
        file
    });
    groups.insert(b.path.clone(), {
        let mut file = HashMap::default();
        file.insert(b.group.into(), b.mask);
        file
    });

    let (locals, globals) = GroupResolver::build(groups);

    // локальные группы
    {
        assert_eq!(locals[&a.path].resolve_mask(b.group), None);
        assert!(locals[&a.path]
            .resolve(b.group, Rule::Deny, Path::new(""))
            .is_err());

        assert_eq!(locals[&a.path].resolve_mask(a.group), Some(a.mask));
        let rule = locals[&a.path]
            .resolve(a.group, Rule::Deny, Path::new(""))
            .unwrap();
        assert_eq!(rule.groups, a.mask);
        assert_eq!(rule.permission, relay::PermissionLevel::Deny as i32);
    }

    // глобальные группы
    {
        assert_eq!(globals.resolve_mask(a.group), Some(a.mask));
        let rule = globals.resolve(a.group, Rule::Deny, Path::new("")).unwrap();
        assert_eq!(rule.groups, a.mask);
        assert_eq!(rule.permission, relay::PermissionLevel::Deny as i32);

        assert_eq!(globals.resolve_mask(b.group), Some(b.mask));
        let rule = globals.resolve(b.group, Rule::Deny, Path::new("")).unwrap();
        assert_eq!(rule.groups, b.mask);
        assert_eq!(rule.permission, relay::PermissionLevel::Deny as i32);
    }
}
