use std::collections::HashMap;

use serde::Deserialize;

use crate::proto::matches::relay::types as relay;
use crate::service::room::error::Error;
use crate::service::room::Rule;

pub type GroupAlias = String;

#[derive(Default, Debug, Deserialize)]
pub struct Groups {
	#[serde(flatten)]
	groups: HashMap<String, u64>,
}

impl Groups {
	pub fn load(self, content: impl AsRef<str>) -> Result<Self, Error> {
		serde_yaml::from_str::<Groups>(content.as_ref()).map_err(Error::GroupParseError)
	}

	pub(crate) fn get_mask(&self, alias: &str) -> Result<u64, Error> {
		self.groups.get(alias).ok_or_else(|| Error::GroupNotFound(alias.to_string())).map(|g| *g)
	}

	pub fn resolve(&self, group: &str, rule: Rule) -> Result<relay::GroupsPermissionRule, Error> {
		let permission = match rule {
			Rule::Deny => relay::PermissionLevel::Deny as i32,
			Rule::ReadOnly => relay::PermissionLevel::Ro as i32,
			Rule::ReadWrite => relay::PermissionLevel::Rw as i32,
		};

		self.groups
			.get(group)
			.copied()
			.map(|groups| relay::GroupsPermissionRule { groups, permission })
			.ok_or_else(|| Error::GroupNotFound(group.to_string()))
	}
}

#[cfg(test)]
#[test]
fn should_resolver() {
	let mut groups = Groups::default();
	groups.groups.insert("groupA".to_string(), 12345);

	let rule = groups.resolve("groupA", Rule::Deny).unwrap();
	assert_eq!(rule.groups, 12345);
	assert_eq!(rule.permission, relay::PermissionLevel::Deny as i32);
}

#[cfg(test)]
#[test]
fn should_load() {
	let content = include_str!("../../../example/groups.yaml");
	let groups = Groups::default().load(content).unwrap();
	assert_eq!(groups.get_mask("red").unwrap(), 1);
	assert_eq!(groups.get_mask("blue").unwrap(), 2);
	assert_eq!(groups.get_mask("bot").unwrap(), 4);
	assert!(groups.get_mask("none").is_err());
}
