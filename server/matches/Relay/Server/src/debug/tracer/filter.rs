use cheetah_matches_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand, S2CCommandWithMeta};
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::UserId;

use crate::debug::tracer::{TracedCommand, UniDirectionCommand};

///
/// Фильтрация сетевых команд на основе правил
///
///
#[derive(Debug)]
pub struct Filter {
	rules: Vec<Vec<Rule>>,
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
	Object(GameObjectId),
}

#[derive(Debug, Eq, PartialEq)]
pub enum RuleCommandDirection {
	S2C,
	C2S,
}

impl Filter {
	pub fn filter(&self, command: &TracedCommand) -> bool {
		self.rules.iter().any(|group| !group.iter().any(|r| !r.filter(command)))
	}
}

impl From<Vec<Vec<Rule>>> for Filter {
	fn from(rules: Vec<Vec<Rule>>) -> Self {
		Self { rules }
	}
}

impl UniDirectionCommand {
	fn get_direction(&self) -> RuleCommandDirection {
		match self {
			UniDirectionCommand::C2S(_) => RuleCommandDirection::C2S,
			UniDirectionCommand::S2C(_) => RuleCommandDirection::S2C,
		}
	}
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			UniDirectionCommand::C2S(command) => match command {
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
			},
			UniDirectionCommand::S2C(command) => match command {
				S2CCommand::Create(_) => Option::None,
				S2CCommand::Created(_) => Option::None,
				S2CCommand::SetLong(command) => Option::Some(command.field_id),
				S2CCommand::SetFloat(command) => Option::Some(command.field_id),
				S2CCommand::SetStruct(command) => Option::Some(command.field_id),
				S2CCommand::Event(command) => Option::Some(command.field_id),
				S2CCommand::Delete(_) => Option::None,
			},
		}
	}

	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			UniDirectionCommand::C2S(command) => match command {
				C2SCommand::Create(command) => Option::Some(command.object_id.clone()),
				C2SCommand::Created(command) => Option::Some(command.object_id.clone()),
				C2SCommand::SetLong(command) => Option::Some(command.object_id.clone()),
				C2SCommand::IncrementLongValue(command) => Option::Some(command.object_id.clone()),
				C2SCommand::CompareAndSetLongValue(command) => Option::Some(command.object_id.clone()),
				C2SCommand::SetFloat(command) => Option::Some(command.object_id.clone()),
				C2SCommand::IncrementFloatCounter(command) => Option::Some(command.object_id.clone()),
				C2SCommand::SetStruct(command) => Option::Some(command.object_id.clone()),
				C2SCommand::Event(command) => Option::Some(command.object_id.clone()),
				C2SCommand::TargetEvent(command) => Option::Some(command.event.object_id.clone()),
				C2SCommand::Delete(command) => Option::Some(command.object_id.clone()),
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
			},
			UniDirectionCommand::S2C(command) => match command {
				S2CCommand::Create(command) => Option::Some(command.object_id.clone()),
				S2CCommand::Created(command) => Option::Some(command.object_id.clone()),
				S2CCommand::SetLong(command) => Option::Some(command.object_id.clone()),
				S2CCommand::SetFloat(command) => Option::Some(command.object_id.clone()),
				S2CCommand::SetStruct(command) => Option::Some(command.object_id.clone()),
				S2CCommand::Event(command) => Option::Some(command.object_id.clone()),
				S2CCommand::Delete(command) => Option::Some(command.object_id.clone()),
			},
		}
	}
}

impl Rule {
	pub fn filter(&self, command: &TracedCommand) -> bool {
		match self {
			Rule::Direction(direction) => *direction == command.network_command.get_direction(),
			Rule::Not(rule) => !rule.filter(command),
			Rule::User(rule_user) => *rule_user == command.user,
			Rule::Template(rule_template) => match &command.template {
				None => false,
				Some(template) => *rule_template == *template,
			},
			Rule::Field(rule_field) => match command.network_command.get_field_id() {
				None => false,
				Some(field_id) => field_id == *rule_field,
			},
			Rule::Object(rule_object_id) => match command.network_command.get_object_id() {
				None => false,
				Some(object_id) => *rule_object_id == object_id,
			},
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::command::event::EventCommand;
	use cheetah_matches_relay_common::commands::command::{C2SCommand, S2CCommand};
	use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::ObjectOwner;
	use cheetah_matches_relay_common::room::UserId;

	use crate::debug::tracer::filter::{Filter, Rule, RuleCommandDirection, TracedCommand, UniDirectionCommand};

	impl TracedCommand {
		pub fn c2s() -> Self {
			Self {
				template: Option::None,
				user: 0,
				network_command: UniDirectionCommand::C2S(C2SCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 0,
					event: Default::default(),
				})),
			}
		}

		pub fn s2c() -> Self {
			Self {
				template: Option::None,
				user: 0,
				network_command: UniDirectionCommand::S2C(S2CCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 0,
					event: Default::default(),
				})),
			}
		}

		pub fn with_user(mut self, user_id: UserId) -> Self {
			self.user = user_id;
			self
		}

		pub fn with_template(mut self, template_id: GameObjectTemplateId) -> Self {
			self.template = Option::Some(template_id);
			self
		}

		pub fn with_field_id(mut self, field_id: FieldId) -> Self {
			match &self.network_command {
				UniDirectionCommand::C2S(command) => {
					if let C2SCommand::Event(mut event_command) = command.clone() {
						event_command.field_id = field_id;
						self.network_command = UniDirectionCommand::C2S(C2SCommand::Event(event_command))
					}
				}
				UniDirectionCommand::S2C(command) => {
					if let S2CCommand::Event(mut event_command) = command.clone() {
						event_command.field_id = field_id;
						self.network_command = UniDirectionCommand::S2C(S2CCommand::Event(event_command));
					}
				}
			}
			self
		}

		pub fn with_object_id(mut self, object_id: GameObjectId) -> Self {
			match &self.network_command {
				UniDirectionCommand::C2S(command) => {
					if let C2SCommand::Event(mut event_command) = command.clone() {
						event_command.object_id = object_id;
						self.network_command = UniDirectionCommand::C2S(C2SCommand::Event(event_command))
					}
				}
				UniDirectionCommand::S2C(command) => {
					if let S2CCommand::Event(mut event_command) = command.clone() {
						event_command.object_id = object_id;
						self.network_command = UniDirectionCommand::S2C(S2CCommand::Event(event_command));
					}
				}
			}
			self
		}
	}

	#[test]
	fn should_filter_by_direction() {
		let filter_c2s = Filter::from(vec![vec![Rule::Direction(RuleCommandDirection::C2S)]]);
		assert!(filter_c2s.filter(&TracedCommand::c2s()));
		assert!(!filter_c2s.filter(&TracedCommand::s2c()));

		let filter_s2c = Filter::from(vec![vec![Rule::Direction(RuleCommandDirection::S2C)]]);
		assert!(filter_s2c.filter(&TracedCommand::s2c()));
		assert!(!filter_s2c.filter(&TracedCommand::c2s()));
	}

	#[test]
	fn should_filter_by_user() {
		let filter = Filter::from(vec![vec![Rule::User(55)]]);
		assert!(filter.filter(&TracedCommand::c2s().with_user(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_user(155)));
	}

	#[test]
	fn should_filter_by_template() {
		let filter = Filter::from(vec![vec![Rule::Template(100)]]);
		assert!(filter.filter(&TracedCommand::c2s().with_template(100)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(200)));
	}

	#[test]
	fn should_filter_by_not() {
		let filter = Filter::from(vec![vec![Rule::Not(Box::new(Rule::Template(100)))]]);
		assert!(filter.filter(&TracedCommand::c2s().with_template(10)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(100)));
	}
	//
	#[test]
	fn should_filter_by_group() {
		let filter = Filter::from(vec![vec![Rule::User(55), Rule::Template(100)]]);
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_user(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_user(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(100)));
	}

	#[test]
	fn should_filter_by_groups() {
		let filter = Filter::from(vec![vec![Rule::User(55), Rule::Template(100)], vec![Rule::User(100), Rule::Template(55)]]);
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_user(55)));
		assert!(filter.filter(&TracedCommand::c2s().with_template(55).with_user(100)));
	}

	#[test]
	fn should_filter_by_field() {
		let filter = Filter::from(vec![vec![Rule::Field(10)]]);
		assert!(filter.filter(&TracedCommand::c2s().with_field_id(10)));
		assert!(filter.filter(&TracedCommand::s2c().with_field_id(10)));
		assert!(!filter.filter(&TracedCommand::c2s().with_field_id(100)));
		assert!(!filter.filter(&TracedCommand::s2c().with_field_id(100)));
	}

	#[test]
	fn should_filter_by_object_id() {
		let filter = Filter::from(vec![vec![Rule::Object(GameObjectId::new(100, ObjectOwner::Root))]]);
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, ObjectOwner::Root))));
		assert!(filter.filter(&TracedCommand::s2c().with_object_id(GameObjectId::new(100, ObjectOwner::Root))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(0, ObjectOwner::Root))));
	}
}
