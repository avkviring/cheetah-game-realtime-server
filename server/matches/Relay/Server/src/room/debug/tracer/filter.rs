use cheetah_matches_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand, S2CCommandWithMeta};
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::room::UserId;

///
/// Фильтрация сетевых команд на основе правил
///
///
pub struct Filter {
	rules: Vec<Vec<Rule>>,
}

impl Filter {
	pub fn filter<T: DirectionCommandGetter + FieldIdCommandGetter>(&self, template: GameObjectTemplateId, user: UserId, command: &T) -> bool {
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
	Field(FieldId),
}

#[derive(Debug, Eq, PartialEq)]
pub enum RuleCommandDirection {
	S2C,
	C2S,
}

impl Rule {
	pub fn filter<T: DirectionCommandGetter + FieldIdCommandGetter>(&self, template: GameObjectTemplateId, user: UserId, command: &T) -> bool {
		match self {
			Rule::Direction(direction) => *direction == command.get_direction(),
			Rule::Not(rule) => !rule.filter(template, user, command),
			Rule::User(rule_user) => *rule_user == user,
			Rule::Template(rule_template) => *rule_template == template,
			Rule::Field(rule_field) => match command.get_field_id() {
				None => false,
				Some(field_id) => field_id == *rule_field,
			},
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

///
/// Получение field_id для команды
///
pub trait FieldIdCommandGetter {
	fn get_field_id(&self) -> Option<FieldId>;
}

impl FieldIdCommandGetter for S2CCommandWithMeta {
	fn get_field_id(&self) -> Option<FieldId> {
		match &self.command {
			S2CCommand::Create(_) => Option::None,
			S2CCommand::Created(_) => Option::None,
			S2CCommand::SetLong(command) => Option::Some(command.field_id),
			S2CCommand::SetFloat(command) => Option::Some(command.field_id),
			S2CCommand::SetStruct(command) => Option::Some(command.field_id),
			S2CCommand::Event(command) => Option::Some(command.field_id),
			S2CCommand::Delete(_) => Option::None,
		}
	}
}

impl FieldIdCommandGetter for C2SCommandWithMeta {
	fn get_field_id(&self) -> Option<FieldId> {
		match &self.command {
			C2SCommand::Create(_) => Option::None,
			C2SCommand::Created(_) => Option::None,
			C2SCommand::SetLong(command) => Option::Some(command.field_id),
			C2SCommand::IncrementLongValue(command) => Option::Some(command.field_id),
			C2SCommand::CompareAndSetLongValue(command) => Option::Some(command.field_id),
			C2SCommand::SetFloat(command) => Option::Some(command.field_id),
			C2SCommand::IncrementFloatCounter(command) => Option::Some(command.field_id),
			C2SCommand::SetStruct(command) => Option::Some(command.field_id),
			C2SCommand::Event(command) => Option::Some(command.field_id),
			C2SCommand::TargetEvent(command) => Option::Some(command.event.field_id),
			C2SCommand::Delete(_) => Option::None,
			C2SCommand::AttachToRoom => Option::None,
			C2SCommand::DetachFromRoom => Option::None,
		}
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

	#[test]
	fn should_filter_by_field() {
		let filter = Filter::from(vec![vec![Rule::Field(10)]]);
		assert!(filter.filter(0, 0, &s2c_event_command(10)));
		assert!(filter.filter(0, 0, &c2s_event_command(10)));
		assert!(!filter.filter(0, 0, &s2c_event_command(100)));
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
