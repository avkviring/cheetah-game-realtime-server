use cheetah_matches_relay_common::commands::command::{C2SCommandWithMeta, S2CCommandWithMeta};
use cheetah_matches_relay_common::constants::GameObjectTemplateId;
use cheetah_matches_relay_common::room::UserId;

///
/// Фильтрация сетевых команд на основе правил
///
///
pub struct Filter {
	rules: Vec<Vec<Rule>>,
}

impl Filter {
	pub fn filter<T: DirectionCommandGetter>(&self, template: GameObjectTemplateId, user: UserId, command: &T) -> bool {
		self.rules.iter().any(|group| !group.iter().any(|r| !r.filter(template, user, command)))
	}
}

impl From<Vec<Vec<Rule>>> for Filter {
	fn from(rules: Vec<Vec<Rule>>) -> Self {
		Self { rules }
	}
}

///
/// Правила фильтрации сетевых команд
///
#[derive(Debug, Eq, PartialEq)]
pub enum Rule {
	Direction(RuleCommandDirection),
	Not(Box<Rule>),
	User(UserId),
	Template(GameObjectTemplateId),
}

#[derive(Debug, Eq, PartialEq)]
pub enum RuleCommandDirection {
	S2C,
	C2S,
}

impl Rule {
	pub fn filter<T: DirectionCommandGetter>(&self, template: GameObjectTemplateId, user: UserId, command: &T) -> bool {
		match self {
			Rule::Direction(direction) => *direction == command.get_direction(),
			Rule::Not(rule) => !rule.filter(template, user, command),
			Rule::User(rule_user) => *rule_user == user,
			Rule::Template(rule_template) => *rule_template == template,
		}
	}
}

///
/// Определение направления для команд
///
pub trait DirectionCommandGetter {
	fn get_direction(&self) -> RuleCommandDirection;
}

impl DirectionCommandGetter for C2SCommandWithMeta {
	fn get_direction(&self) -> RuleCommandDirection {
		RuleCommandDirection::C2S
	}
}

impl DirectionCommandGetter for S2CCommandWithMeta {
	fn get_direction(&self) -> RuleCommandDirection {
		RuleCommandDirection::S2C
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::command::event::EventCommand;
	use cheetah_matches_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_matches_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
	use cheetah_matches_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand, S2CCommandWithMeta};
	use cheetah_matches_relay_common::constants::FieldId;

	use crate::room::debug::tracer::filter::*;

	#[test]
	fn should_filter_by_direction() {
		let filter_c2s = Filter::from(vec![vec![Rule::Direction(RuleCommandDirection::C2S)]]);
		assert!(filter_c2s.filter(0, 0, &c2s_event_command(0)));
		assert!(!filter_c2s.filter(0, 0, &s2c_event_command(0)));

		let filter_s2c = Filter::from(vec![vec![Rule::Direction(RuleCommandDirection::S2C)]]);
		assert!(filter_s2c.filter(0, 0, &s2c_event_command(0)));
		assert!(!filter_s2c.filter(0, 0, &c2s_event_command(0)));
	}

	#[test]
	fn should_filter_by_user() {
		let filter = Filter::from(vec![vec![Rule::User(55)]]);
		assert!(filter.filter(0, 55, &s2c_event_command(0)));
		assert!(!filter.filter(0, 155, &s2c_event_command(0)));
	}

	#[test]
	fn should_filter_by_template() {
		let filter = Filter::from(vec![vec![Rule::Template(100)]]);
		assert!(filter.filter(100, 0, &s2c_event_command(0)));
		assert!(!filter.filter(10, 0, &s2c_event_command(0)));
	}

	#[test]
	fn should_filter_by_not() {
		let filter = Filter::from(vec![vec![Rule::Not(Box::new(Rule::Template(100)))]]);
		assert!(filter.filter(10, 0, &s2c_event_command(0)));
		assert!(!filter.filter(100, 0, &s2c_event_command(0)));
	}

	#[test]
	fn should_filter_by_group() {
		let filter = Filter::from(vec![vec![Rule::User(55), Rule::Template(100)]]);
		assert!(filter.filter(100, 55, &s2c_event_command(0)));
		assert!(!filter.filter(0, 55, &s2c_event_command(0)));
		assert!(!filter.filter(100, 0, &s2c_event_command(0)));
	}

	#[test]
	fn should_filter_by_groups() {
		let filter = Filter::from(vec![vec![Rule::User(55), Rule::Template(100)], vec![Rule::User(100), Rule::Template(55)]]);
		assert!(filter.filter(100, 55, &s2c_event_command(0)));
		assert!(filter.filter(55, 100, &s2c_event_command(0)));
	}

	fn s2c_event_command(field_id: FieldId) -> S2CCommandWithMeta {
		S2CCommandWithMeta {
			meta: S2CMetaCommandInformation {
				user_id: 0,
				timestamp: 0,
				source_object: None,
			},
			command: S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id,
				event: Default::default(),
			}),
		}
	}

	fn c2s_event_command(field_id: FieldId) -> C2SCommandWithMeta {
		C2SCommandWithMeta {
			meta: C2SMetaCommandInformation {
				timestamp: 0,
				source_object: None,
			},
			command: C2SCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id,
				event: Default::default(),
			}),
		}
	}
}
