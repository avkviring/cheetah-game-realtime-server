use std::collections::HashMap;
use std::io::Read;

use log::Level;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::commands::command::{C2SCommand, S2CCommand};
use cheetah_relay_common::constants::FieldIDType;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::types::FieldType;
use crate::room::RoomId;

///
/// Вывод отладочной информации по командам с клиента/сервера с учетом правил фильтрации.
/// Для отображения информации используется log::info
///
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTracer {
	default: Action,
	///
	/// Правила применяются последовательно до первого срабатывания
	///
	rules: Vec<Rule>,

	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
	action: Action,
	command: Option<Command>,
	direction: Option<Direction>,
	field_type: Option<FieldType>,
	field_id: Option<FieldIDType>,
	user: Option<UserPublicKey>,

	#[serde(flatten)]
	pub unmapping: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Action {
	#[serde(rename = "allow")]
	Allow,
	#[serde(rename = "deny")]
	Deny,
}

///
/// Направление команды (server->client, client->server)
///
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Direction {
	#[serde(rename = "sc")]
	SC,
	#[serde(rename = "cs")]
	CS,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Command {
	Create,
	Created,
	SetLong,
	IncrementLongValue,
	CompareAndSetLongValue,
	SetFloat,
	IncrementFloatValue,
	SetStruct,
	Event,
	Delete,
	AttachToRoom,
}

impl Rule {
	fn is_match(
		&self,
		user_public_key: UserPublicKey,
		direction: &Direction,
		command: &Command,
		field_type: &Option<FieldType>,
		field: &Option<FieldIDType>,
	) -> bool {
		EqualResult::NotEqual != is_match_with_option(&self.field_type, field_type)
			&& EqualResult::NotEqual != is_match_with_option(&self.field_id, field)
			&& EqualResult::NotEqual != is_match(&self.user, &user_public_key)
			&& EqualResult::NotEqual != is_match(&self.direction, direction)
			&& EqualResult::NotEqual != is_match(&self.command, command)
	}
}

#[derive(PartialEq)]
enum EqualResult {
	Skip,
	Equal,
	NotEqual,
}

///
///  Возвращает default если поле для сравнение не задано, если поле задано - то результат сравнения
///  
fn is_match<T: PartialEq>(a: &Option<T>, b: &T) -> EqualResult {
	match a {
		None => EqualResult::Skip,
		Some(a) => {
			if a == b {
				EqualResult::Equal
			} else {
				EqualResult::NotEqual
			}
		}
	}
}

///
/// Возвращает default если поле для сравнение не задано, если поле задано - то результат сравнения
///
fn is_match_with_option<T: PartialEq>(a: &Option<T>, b: &Option<T>) -> EqualResult {
	match a {
		None => EqualResult::Skip,
		Some(a) => match b {
			None => EqualResult::NotEqual,
			Some(b) => {
				if a == b {
					EqualResult::Equal
				} else {
					EqualResult::NotEqual
				}
			}
		},
	}
}

#[derive(Debug)]
pub enum CommandTraceError {
	YamlParserError(serde_yaml::Error),
	YamlContainsUnmappingFields(Vec<String>),
}

impl CommandTracer {
	pub fn load_from_file(path: String) -> Result<Self, CommandTraceError> {
		let mut file = std::fs::File::open(path).unwrap();
		let mut content = String::default();
		file.read_to_string(&mut content).unwrap();
		let tracer: CommandTracer = serde_yaml::from_str(content.as_str()).map_err(|e| CommandTraceError::YamlParserError(e))?;
		tracer.validate()
	}

	pub fn validate(self) -> Result<Self, CommandTraceError> {
		let mut unmapping = Vec::new();
		self.unmapping.iter().for_each(|(key, _value)| unmapping.push(key.clone()));
		for rule in &self.rules {
			rule.unmapping.iter().for_each(|(key, _value)| unmapping.push(format!("rule/{}", key)));
		}
		if unmapping.is_empty() {
			Result::Ok(self)
		} else {
			Result::Err(CommandTraceError::YamlContainsUnmappingFields(unmapping))
		}
	}

	///
	/// Создать трейсер для отображения всех событий
	///
	pub fn new_with_allow_all() -> Self {
		Self {
			default: Action::Allow,
			rules: vec![],
			unmapping: Default::default(),
		}
	}

	pub fn new_with_deny_all() -> Self {
		Self {
			default: Action::Deny,
			rules: vec![],
			unmapping: Default::default(),
		}
	}

	fn is_allow(
		&self,
		user: UserPublicKey,
		direction: Direction,
		command: Command,
		field_type: Option<FieldType>,
		field: Option<FieldIDType>,
	) -> bool {
		let action = match self.rules.iter().find(|p| p.is_match(user, &direction, &command, &field_type, &field)) {
			None => &self.default,
			Some(rule) => &rule.action,
		};
		*action == Action::Allow
	}

	pub fn on_s2c_command(&self, room_id: RoomId, user_public_key: UserPublicKey, command: &S2CCommand) {
		if !(log::log_enabled!(Level::Info)) {
			return;
		}

		let info = match command {
			S2CCommand::Create(_c) => (Command::Create, Option::None, Option::None),
			S2CCommand::Created(_c) => (Command::Created, Option::None, Option::None),
			S2CCommand::SetLong(c) => (Command::SetLong, Option::Some(FieldType::Long), Option::Some(c.field_id)),
			S2CCommand::SetFloat(c) => (Command::SetFloat, Option::Some(FieldType::Float), Option::Some(c.field_id)),
			S2CCommand::SetStruct(c) => (Command::SetStruct, Option::Some(FieldType::Structure), Option::Some(c.field_id)),
			S2CCommand::Event(c) => (Command::Event, Option::Some(FieldType::Event), Option::Some(c.field_id)),
			S2CCommand::Delete(_c) => (Command::Delete, Option::None, Option::None),
		};

		if self.is_allow(user_public_key, Direction::SC, info.0, info.1, info.2) {
			log::info!("[room({:?})] s -> u({:?}) {:?}", room_id, user_public_key, command);
		}
	}
	pub fn on_c2s_command(&self, room_id: RoomId, user_public_key: UserPublicKey, command: &C2SCommand) {
		if !(log::log_enabled!(Level::Info)) {
			return;
		}

		let info = match command {
			C2SCommand::Create(_) => (Command::Create, Option::None, Option::None),
			C2SCommand::Created(_) => (Command::Created, Option::None, Option::None),
			C2SCommand::SetLong(c) => (Command::SetLong, Option::Some(FieldType::Long), Option::Some(c.field_id)),
			C2SCommand::SetFloat(c) => (Command::SetFloat, Option::Some(FieldType::Float), Option::Some(c.field_id)),
			C2SCommand::SetStruct(c) => (Command::SetStruct, Option::Some(FieldType::Structure), Option::Some(c.field_id)),
			C2SCommand::Event(c) => (Command::Event, Option::Some(FieldType::Event), Option::Some(c.field_id)),
			C2SCommand::Delete(_) => (Command::Delete, Option::None, Option::None),
			C2SCommand::IncrementLongValue(c) => (Command::IncrementLongValue, Option::Some(FieldType::Long), Option::Some(c.field_id)),
			C2SCommand::CompareAndSetLongValue(c) => (Command::CompareAndSetLongValue, Option::Some(FieldType::Long), Option::Some(c.field_id)),
			C2SCommand::IncrementFloatCounter(c) => (Command::IncrementFloatValue, Option::Some(FieldType::Float), Option::Some(c.field_id)),
			C2SCommand::AttachToRoom => (Command::AttachToRoom, Option::None, Option::None),
		};
		if self.is_allow(user_public_key, Direction::CS, info.0, info.1, info.2) {
			log::info!("[room({:?})] u({:?}) -> s {:?}", room_id, user_public_key, command);
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::room::debug::tracer::{Action, Command, CommandTraceError, CommandTracer, Direction, Rule};
	use crate::room::types::FieldType;

	#[test]
	#[allow(dead_code)]
	pub fn export() {
		let tracer = CommandTracer::new_with_allow_all();
		let content = serde_yaml::to_string(&tracer).unwrap();
		println!("{}", content);
	}

	#[test]
	pub fn should_match_skip_all_fields() {
		let rule = Rule {
			action: Action::Allow,
			command: None,
			direction: None,
			field_type: None,
			field_id: None,
			user: None,
			unmapping: Default::default(),
		};

		assert!(rule.is_match(1, &Direction::CS, &Command::Created, &Option::None, &Option::None))
	}

	#[test]
	pub fn should_match_equal_all_fields() {
		let rule = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Created),
			direction: Option::Some(Direction::CS),
			field_type: Option::Some(FieldType::Long),
			field_id: Option::Some(55),
			user: Option::Some(1),
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::CS, &Command::Created, &Option::Some(FieldType::Long), &Option::Some(55)))
	}

	#[test]
	pub fn should_match_command() {
		let rule = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Created),
			direction: None,
			field_type: None,
			field_id: None,
			user: None,
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::CS, &Command::Created, &Option::None, &Option::None));
		assert!(!rule.is_match(
			1,
			&Direction::CS,
			&Command::SetStruct,
			&Option::Some(FieldType::Structure),
			&Option::Some(55)
		))
	}

	#[test]
	pub fn should_match_direction() {
		let rule = Rule {
			action: Action::Allow,
			command: None,
			direction: Some(Direction::SC),
			field_type: None,
			field_id: None,
			user: None,
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::SC, &Command::Created, &Option::None, &Option::None));
		assert!(!rule.is_match(1, &Direction::CS, &Command::Created, &Option::None, &Option::None));
	}

	#[test]
	pub fn should_match_field_type() {
		let rule = Rule {
			action: Action::Allow,
			command: None,
			direction: None,
			field_type: Some(FieldType::Event),
			field_id: None,
			user: None,
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::SC, &Command::Created, &Option::Some(FieldType::Event), &Option::None));
		assert!(!rule.is_match(1, &Direction::SC, &Command::Created, &Option::None, &Option::None));
		assert!(!rule.is_match(1, &Direction::SC, &Command::Created, &Option::Some(FieldType::Structure), &Option::None));
	}

	#[test]
	pub fn should_match_field() {
		let rule = Rule {
			action: Action::Allow,
			command: None,
			direction: None,
			field_type: None,
			field_id: Some(55),
			user: None,
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::SC, &Command::Created, &Option::None, &Option::Some(55)));
		assert!(!rule.is_match(1, &Direction::SC, &Command::Created, &Option::None, &Option::None));
		assert!(!rule.is_match(1, &Direction::SC, &Command::Created, &Option::None, &Option::Some(33)));
	}

	#[test]
	pub fn should_match_user() {
		let rule = Rule {
			action: Action::Allow,
			command: None,
			direction: None,
			field_type: None,
			field_id: None,
			user: Some(1),
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::CS, &Command::SetFloat, &Option::Some(FieldType::Long), &Option::Some(55)));
		assert!(!rule.is_match(2, &Direction::CS, &Command::SetFloat, &Option::Some(FieldType::Long), &Option::Some(55)))
	}

	#[test]
	pub fn should_not_match_when_one_field_is_not_equal() {
		let rule = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Event),
			direction: Option::Some(Direction::CS),
			field_type: Option::Some(FieldType::Long),
			field_id: Option::Some(55),
			user: Option::Some(1),
			unmapping: Default::default(),
		};
		assert!(!rule.is_match(1, &Direction::CS, &Command::Created, &Option::Some(FieldType::Long), &Option::Some(55)))
	}

	#[test]
	pub fn should_match_when_one_field_is_skip() {
		let rule = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Created),
			direction: None,
			field_type: Option::Some(FieldType::Long),
			field_id: Option::Some(55),
			user: Option::Some(1),
			unmapping: Default::default(),
		};
		assert!(rule.is_match(1, &Direction::CS, &Command::Created, &Option::Some(FieldType::Long), &Option::Some(55)))
	}

	#[test]
	pub fn should_tracer_match() {
		let rule_a = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Created),
			direction: None,
			field_type: Option::Some(FieldType::Long),
			field_id: Option::Some(55),
			user: Option::Some(1),
			unmapping: Default::default(),
		};
		let rule_b = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Event),
			direction: None,
			field_type: None,
			field_id: None,
			user: None,
			unmapping: Default::default(),
		};
		let tracer = CommandTracer {
			default: Action::Deny,
			rules: vec![rule_a.clone(), rule_b.clone()],
			unmapping: Default::default(),
		};

		assert!(tracer.is_allow(1, Direction::CS, Command::Created, Option::Some(FieldType::Long), Option::Some(55)));
		assert!(tracer.is_allow(1, Direction::CS, Command::Event, Option::Some(FieldType::Long), Option::Some(55)));
		assert!(!tracer.is_allow(1, Direction::CS, Command::SetFloat, Option::Some(FieldType::Long), Option::Some(55)));

		let tracer = CommandTracer {
			default: Action::Allow,
			rules: vec![rule_a, rule_b],
			unmapping: Default::default(),
		};

		assert!(tracer.is_allow(1, Direction::CS, Command::SetFloat, Option::Some(FieldType::Long), Option::Some(55)));
	}

	#[test]
	pub fn should_validate() {
		let mut rule = Rule {
			action: Action::Allow,
			command: Option::Some(Command::Created),
			direction: None,
			field_type: Option::Some(FieldType::Long),
			field_id: Option::Some(55),
			user: Option::Some(1),
			unmapping: Default::default(),
		};

		rule.unmapping.insert("wrong_field".to_string(), serde_yaml::Value::default());

		let mut tracer = CommandTracer {
			default: Action::Deny,
			rules: vec![rule],
			unmapping: Default::default(),
		};
		tracer.unmapping.insert("wrong_field".to_string(), serde_yaml::Value::default());

		assert!(matches!(
			tracer.validate(),
			Result::Err(CommandTraceError::YamlContainsUnmappingFields(fields))
			if fields[0] == "wrong_field"
			&& fields[1] == "rule/wrong_field"

		))
	}
}
