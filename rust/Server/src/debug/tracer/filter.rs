use cheetah_common::room::field::{FieldId, FieldType};
use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_protocol::RoomMemberId;

use crate::debug::tracer::{TracedBothDirectionCommand, TracedCommand};

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
	Member(RoomMemberId),
	Template(GameObjectTemplateId),
	Field(FieldId),
	RoomOwner,
	MemberOwner(RoomMemberId),
	ObjectId(u32),
	True,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RuleCommandDirection {
	S2C,
	C2S,
}

impl Filter {
	#[must_use]
	pub fn new(rule: Rule) -> Self {
		Self { rule }
	}
	#[must_use]
	pub fn filter(&self, command: &TracedCommand) -> bool {
		self.rule.filter(command)
	}
}

impl TracedBothDirectionCommand {
	fn get_direction(&self) -> RuleCommandDirection {
		match self {
			TracedBothDirectionCommand::C2S(_) => RuleCommandDirection::C2S,
			TracedBothDirectionCommand::S2C(_) => RuleCommandDirection::S2C,
		}
	}
	pub(crate) fn get_field_type(&self) -> Option<FieldType> {
		match self {
			TracedBothDirectionCommand::C2S(command) => command.get_field_type(),
			TracedBothDirectionCommand::S2C(command) => command.get_field_type(),
		}
	}

	pub(crate) fn get_field_id(&self) -> Option<FieldId> {
		match self {
			TracedBothDirectionCommand::C2S(command) => command.get_field_id(),
			TracedBothDirectionCommand::S2C(command) => command.get_field_id(),
		}
	}

	pub(crate) fn get_object_id(&self) -> Option<GameObjectId> {
		match self {
			TracedBothDirectionCommand::C2S(command) => command.get_object_id(),
			TracedBothDirectionCommand::S2C(command) => command.get_object_id(),
		}
	}
}

impl Rule {
	#[must_use]
	pub fn filter(&self, command: &TracedCommand) -> bool {
		match self {
			Rule::Direction(direction) => *direction == command.network_command.get_direction(),
			Rule::Not(rule) => !rule.filter(command),
			Rule::Member(member_id) => *member_id == command.member,
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
				Some(object_id) => match object_id.get_owner() {
					GameObjectOwner::Room => true,
					GameObjectOwner::Member(_) => false,
				},
			},
			Rule::MemberOwner(member_id) => match command.network_command.get_object_id() {
				None => false,
				Some(object_id) => match object_id.get_owner() {
					GameObjectOwner::Room => false,
					GameObjectOwner::Member(object_member_id) => object_member_id == *member_id,
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
	use cheetah_common::commands::c2s::C2SCommand;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::event::EventCommand;
	use cheetah_common::room::field::FieldId;
	use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};
	use cheetah_common::room::owner::GameObjectOwner;
	use cheetah_protocol::RoomMemberId;

	use crate::debug::tracer::filter::{Filter, Rule, RuleCommandDirection, TracedBothDirectionCommand, TracedCommand};

	impl TracedCommand {
		#[must_use]
		pub fn c2s() -> Self {
			Self {
				time: 0.0,
				template: None,
				member: 0,
				network_command: TracedBothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 0,
					event: Default::default(),
				})),
			}
		}

		#[must_use]
		pub fn s2c() -> Self {
			Self {
				time: 0.0,
				template: None,
				member: 0,
				network_command: TracedBothDirectionCommand::S2C(S2CCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 0,
					event: Default::default(),
				})),
			}
		}

		#[must_use]
		pub fn with_member(mut self, member_id: RoomMemberId) -> Self {
			self.member = member_id;
			self
		}

		#[must_use]
		pub fn with_template(mut self, template_id: GameObjectTemplateId) -> Self {
			self.template = Some(template_id);
			self
		}

		#[must_use]
		pub fn with_field_id(mut self, field_id: FieldId) -> Self {
			match &self.network_command {
				TracedBothDirectionCommand::C2S(command) => {
					if let C2SCommand::Event(mut event_command) = command.clone() {
						event_command.field_id = field_id;
						self.network_command = TracedBothDirectionCommand::C2S(C2SCommand::Event(event_command));
					}
				}
				TracedBothDirectionCommand::S2C(command) => {
					if let S2CCommand::Event(mut event_command) = command.clone() {
						event_command.field_id = field_id;
						self.network_command = TracedBothDirectionCommand::S2C(S2CCommand::Event(event_command));
					}
				}
			}
			self
		}

		#[must_use]
		pub fn with_object_id(mut self, object_id: GameObjectId) -> Self {
			match &self.network_command {
				TracedBothDirectionCommand::C2S(command) => {
					if let C2SCommand::Event(mut event_command) = command.clone() {
						event_command.object_id = object_id;
						self.network_command = TracedBothDirectionCommand::C2S(C2SCommand::Event(event_command));
					}
				}
				TracedBothDirectionCommand::S2C(command) => {
					if let S2CCommand::Event(mut event_command) = command.clone() {
						event_command.object_id = object_id;
						self.network_command = TracedBothDirectionCommand::S2C(S2CCommand::Event(event_command));
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
	fn should_filter_by_member() {
		let filter = Filter::new(Rule::Member(55));
		assert!(filter.filter(&TracedCommand::c2s().with_member(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_member(155)));
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
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::Member(100)))));
	}

	#[test]
	fn should_filter_by_member_owner() {
		let filter = Filter::new(Rule::MemberOwner(55));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::Member(55)))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Room))));
	}

	#[test]
	fn should_filter_by_object_id() {
		let filter = Filter::new(Rule::ObjectId(100));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Member(55)))));
		assert!(filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(100, GameObjectOwner::Room))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(70, GameObjectOwner::Member(55)))));
		assert!(!filter.filter(&TracedCommand::c2s().with_object_id(GameObjectId::new(50, GameObjectOwner::Room))));
	}

	#[test]
	fn should_filter_or() {
		let filter = Filter::new(Rule::OrRule(vec![Rule::Template(100), Rule::Template(55), Rule::Member(55)]));
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_member(55)));
		assert!(filter.filter(&TracedCommand::c2s().with_template(55).with_member(100)));
	}

	#[test]
	fn should_filter_and() {
		let filter = Filter::new(Rule::AndRule(vec![Rule::Template(100), Rule::Member(55)]));
		assert!(filter.filter(&TracedCommand::c2s().with_template(100).with_member(55)));
		assert!(!filter.filter(&TracedCommand::c2s().with_template(55).with_member(100)));
	}
}
