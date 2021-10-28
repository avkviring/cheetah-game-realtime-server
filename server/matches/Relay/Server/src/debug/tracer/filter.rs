use cheetah_matches_relay_common::commands::command::{C2SCommand, S2CCommand, S2CCommandWithMeta};
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::UserId;

use crate::debug::tracer::{TracedCommand, UniDirectionCommand};
use crate::room::types::FieldType;

///
/// Фильтрация сетевых команд на основе правил
///
///
#[derive(Debug)]
pub struct Filter {
	rule: Rule,
}

///
/// Правила фильтрации сетевых команд
///
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Rule {
	OrRule(Vec<Rule>),
	AndRule(Vec<Rule>),
	Direction(RuleCommandDirection),
	Not(Box<Rule>),
	User(UserId),
	Template(GameObjectTemplateId),
	Field(FieldId),
	RoomOwner,
	UserOwner(UserId),
	ObjectId(u32),
	True,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RuleCommandDirection {
	S2C,
	C2S,
}

impl Filter {
	pub fn new(rule: Rule) -> Self {
		Self { rule }
	}
	pub fn filter(&self, command: &TracedCommand) -> bool {
		self.rule.filter(command)
	}
}

impl UniDirectionCommand {
	fn get_direction(&self) -> RuleCommandDirection {
		match self {
			UniDirectionCommand::C2S(_) => RuleCommandDirection::C2S,
			UniDirectionCommand::S2C(_) => RuleCommandDirection::S2C,
		}
	}
	pub fn get_field_type(&self) -> Option<FieldType> {
		match self {
			UniDirectionCommand::C2S(command) => match command {
				C2SCommand::Create(_) => Option::None,
				C2SCommand::Created(_) => Option::None,
				C2SCommand::SetLong(_) => Option::Some(FieldType::Long),
				C2SCommand::IncrementLongValue(_) => Option::Some(FieldType::Long),
				C2SCommand::CompareAndSetLongValue(_) => Option::Some(FieldType::Long),
				C2SCommand::SetFloat(_) => Option::Some(FieldType::Double),
				C2SCommand::IncrementFloatCounter(_) => Option::Some(FieldType::Double),
				C2SCommand::SetStruct(_) => Option::Some(FieldType::Structure),
				C2SCommand::Event(_) => Option::Some(FieldType::Event),
				C2SCommand::TargetEvent(_) => Option::Some(FieldType::Event),
				C2SCommand::Delete(_) => Option::None,
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
			},
			UniDirectionCommand::S2C(command) => match command {
				S2CCommand::Create(_) => Option::None,
				S2CCommand::Created(_) => Option::None,
				S2CCommand::SetLong(_) => Option::Some(FieldType::Long),
				S2CCommand::SetFloat(_) => Option::Some(FieldType::Double),
				S2CCommand::SetStruct(_) => Option::Some(FieldType::Structure),
				S2CCommand::Event(_) => Option::Some(FieldType::Event),
				S2CCommand::Delete(_) => Option::None,
			},
		}
	}
	pub fn get_field_id(&self) -> Option<FieldId> {
		match self {
			UniDirectionCommand::C2S(command) => match command {
				C2SCommand::Create(_) => Option::None,
				C2SCommand::Created(_) => Option::None,
				C2SCommand::SetLong(command) => Some(command.field_id),
				C2SCommand::IncrementLongValue(command) => Some(command.field_id),
				C2SCommand::CompareAndSetLongValue(command) => Some(command.field_id),
				C2SCommand::SetFloat(command) => Some(command.field_id),
				C2SCommand::IncrementFloatCounter(command) => Some(command.field_id),
				C2SCommand::SetStruct(command) => Some(command.field_id),
				C2SCommand::Event(command) => Some(command.field_id),
				C2SCommand::TargetEvent(command) => Some(command.event.field_id),
				C2SCommand::Delete(_) => Option::None,
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
			},
			UniDirectionCommand::S2C(command) => match command {
				S2CCommand::Create(_) => Option::None,
				S2CCommand::Created(_) => Option::None,
				S2CCommand::SetLong(command) => Some(command.field_id),
				S2CCommand::SetFloat(command) => Some(command.field_id),
				S2CCommand::SetStruct(command) => Some(command.field_id),
				S2CCommand::Event(command) => Some(command.field_id),
				S2CCommand::Delete(_) => Option::None,
			},
		}
	}

	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			UniDirectionCommand::C2S(command) => match command {
				C2SCommand::Create(command) => Some(command.object_id.clone()),
				C2SCommand::Created(command) => Some(command.object_id.clone()),
				C2SCommand::SetLong(command) => Some(command.object_id.clone()),
				C2SCommand::IncrementLongValue(command) => Some(command.object_id.clone()),
				C2SCommand::CompareAndSetLongValue(command) => Some(command.object_id.clone()),
				C2SCommand::SetFloat(command) => Some(command.object_id.clone()),
				C2SCommand::IncrementFloatCounter(command) => Some(command.object_id.clone()),
				C2SCommand::SetStruct(command) => Some(command.object_id.clone()),
				C2SCommand::Event(command) => Some(command.object_id.clone()),
				C2SCommand::TargetEvent(command) => Some(command.event.object_id.clone()),
				C2SCommand::Delete(command) => Some(command.object_id.clone()),
				C2SCommand::AttachToRoom => Option::None,
				C2SCommand::DetachFromRoom => Option::None,
			},
			UniDirectionCommand::S2C(command) => match command {
				S2CCommand::Create(command) => Some(command.object_id.clone()),
				S2CCommand::Created(command) => Some(command.object_id.clone()),
				S2CCommand::SetLong(command) => Some(command.object_id.clone()),
				S2CCommand::SetFloat(command) => Some(command.object_id.clone()),
				S2CCommand::SetStruct(command) => Some(command.object_id.clone()),
				S2CCommand::Event(command) => Some(command.object_id.clone()),
				S2CCommand::Delete(command) => Some(command.object_id.clone()),
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
			Rule::OrRule(rules) => rules.iter().any(|r| r.filter(command)),
			Rule::AndRule(rules) => rules.iter().all(|r| r.filter(command)),
			Rule::RoomOwner => match command.network_command.get_object_id() {
				None => false,
				Some(object_id) => match object_id.owner {
					GameObjectOwner::Room => true,
					GameObjectOwner::User(_) => false,
				},
			},
			Rule::UserOwner(user) => match command.network_command.get_object_id() {
				None => false,
				Some(object_id) => match object_id.owner {
					GameObjectOwner::Room => false,
					GameObjectOwner::User(object_user) => object_user == *user,
				},
			},
			Rule::ObjectId(object_id) => match command.network_command.get_object_id() {
				None => false,
				Some(game_object_id) => game_object_id.id == *object_id,
			},
			Rule::True => true,
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::command::event::EventCommand;
	use cheetah_matches_relay_common::commands::command::{C2SCommand, S2CCommand};
	use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::UserId;

	use crate::debug::tracer::filter::{Filter, Rule, RuleCommandDirection, TracedCommand, UniDirectionCommand};

	impl TracedCommand {
		pub fn c2s() -> Self {
			Self {
				time: 0.0,
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
				time: 0.0,
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
			self.template = Some(template_id);
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
		let filter_c2s = Filter::new(Rule::Direction(RuleCommandDirection::C2S));
		assert!(filter_c2s.filter(&TracedCommand::c2s()));
		assert!(!filter_c2s.filter(&TracedCommand::s2c()));

		let filter_s2c = Filter::new(Rule::Direction(RuleCommandDirection::S2C));
		assert!(filter_s2c.filter(&TracedCommand::s2c()));
		assert!(!filter_s2c.filter(&TracedCommand::c2s()));
	}

	#[test]
	fn should_filter_by_user() {
		let filter = Filter::new(Rule::User(55));
		assert!(filter.filter(&TracedCommand::c2s().with_user(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_user(155)));
	}

	#[test]
	fn should_filter_by_template() {
		let filter = Filter::new(Rule::Template(100));
		assert!(filter.filter(&TracedCommand::c2s().with_template(100)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(200)));
	}

	#[test]
	fn should_filter_by_not() {
		let filter = Filter::new(Rule::Not(Box::new(Rule::Template(100))));
		assert!(filter.filter(&TracedCommand::c2s().with_template(10)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(100)));
	}

	#[test]
	fn should_filter_by_field() {
		let filter = Filter::new(Rule::Field(10));
		assert!(filter.filter(&TracedCommand::c2s().with_field_id(10)));
		assert!(filter.filter(&TracedCommand::s2c().with_field_id(10)));
		assert!(!filter.filter(&TracedCommand::c2s().with_field_id(100)));
		assert!(!filter.filter(&TracedCommand::s2c().with_field_id(100)));
	}

	#[test]
	fn should_filter_by_room_owner() {
		let filter = Filter::new(Rule::RoomOwner);
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Room))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::User(100)))));
	}
	#[test]
	fn should_filter_by_user_owner() {
		let filter = Filter::new(Rule::UserOwner(55));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::User(55)))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Room))));
	}
	#[test]
	fn should_filter_by_object_id() {
		let filter = Filter::new(Rule::ObjectId(100));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::User(55)))));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Room))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(70, GameObjectOwner::User(55)))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::Room))));
	}

	#[test]
	fn should_filter_or() {
		let filter = Filter::new(Rule::OrRule(vec![Rule::Template(100), Rule::Template(55), Rule::User(55)]));
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_user(55)));
		assert!(filter.filter(&TracedCommand::c2s().with_template(55).with_user(100)));
	}

	#[test]
	fn should_filter_and() {
		let filter = Filter::new(Rule::AndRule(vec![Rule::Template(100), Rule::User(55)]));
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_user(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(55).with_user(100)));
	}
}
