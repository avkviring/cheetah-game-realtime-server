use cheetah_matches_relay_common::constants::GameObjectTemplateId;
use cheetah_matches_relay_common::room::UserId;

pub mod parser;

///
/// Сервис визуализации потока сетевых команд для отладки
/// adr/matches/0002-relay-debug-commands-flow-in-unity.md
///
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
