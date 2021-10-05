use pom::parser::Parser;
use pom::parser::*;

use crate::debug::tracer::filter::{Rule, RuleCommandDirection};
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::ObjectOwner;

///
/// Парсер текстового фильтра в структуру Rule
/// todo - неплохо бы сделать понятные сообщения об ошибках
///
pub fn parser<'a>() -> Parser<'a, u8, Vec<Vec<Rule>>> {
	let list = list(seq(b"(") * rules_group() - seq(b")"), space()) - end();
	let single = (rules_group() - end()).map(|v| vec![v]);
	single | list
}

///
/// Набор правил, без скобок - user=id, template=id, ...
///  
fn rules_group<'a>() -> Parser<'a, u8, Vec<Rule>> {
	list(call(rules_with_not), space() * seq(b",") * space())
}

///
/// Любое количество пробельных символов
///
fn space<'a>() -> Parser<'a, u8, ()> {
	one_of(b" \t\r\n").repeat(0..).discard()
}

///
/// Идентификатор (u64)
fn id<'a>() -> Parser<'a, u8, u64> {
	(space() + one_of(b"0123456789").repeat(0..) - space()).map(|v| std::str::from_utf8(&v.1).unwrap().parse().unwrap())
}

enum Op {
	EQUALS,
	NOT,
}

///
/// поле вида name=id
///
fn field<'a>(name: &'a str) -> Parser<'a, u8, Op> {
	let name = space() * (seq(name.as_bytes()).discard() | sym(name.as_bytes()[0]).discard()) * space();
	let op = seq(b"=").map(|_| Op::EQUALS) | seq(b"!=").map(|_| Op::NOT);
	(name + op).map(|v| v.1)
}

///
/// Правило с отрицанием - !s2c, !user=55, ...
///
fn rules_with_not<'a>() -> Parser<'a, u8, Rule> {
	(sym(b'!') * rules()).map(|v| Rule::Not(Box::new(v))) | rules()
}

///
/// Конечные правила
///
fn rules<'a>() -> Parser<'a, u8, Rule> {
	seq(b"s2c").map(|_| Rule::Direction(RuleCommandDirection::S2C))
		| seq(b"c2s").map(|_| Rule::Direction(RuleCommandDirection::C2S))
		| (field("user") + id()).map(|(op, id)| apply_op(op, Rule::User(id as u16)))
		| (field("template") + id()).map(|(op, id)| apply_op(op, Rule::Template(id as u16)))
		| (field("field") + id()).map(|(op, id)| apply_op(op, Rule::Field(id as u16)))
		| (field("object") - seq(b"root(") + id() - seq(b")"))
			.map(|(op, id)| apply_op(op, Rule::Object(GameObjectId::new(id as u32, ObjectOwner::Root))))
		| (field("object") - seq(b"user(") + id() - one_of(b",") + id() - seq(b")"))
			.map(|((op, user), id)| apply_op(op, Rule::Object(GameObjectId::new(id as u32, ObjectOwner::User(user as u16)))))
}

///
/// Применить операцию для условия
///
fn apply_op(op: Op, rule: Rule) -> Rule {
	match op {
		Op::EQUALS => rule,
		Op::NOT => Rule::Not(Box::new(rule)),
	}
}

#[cfg(test)]
mod test {
	use crate::debug::tracer::filter::{Rule, RuleCommandDirection};
	use crate::debug::tracer::parser::parser;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::ObjectOwner;

	#[test]
	fn should_parse_single_group() {
		let query = "c2s,s2c";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![vec![
				Rule::Direction(RuleCommandDirection::C2S),
				Rule::Direction(RuleCommandDirection::S2C)
			]]
		)
	}

	#[test]
	fn should_parse_single_group_with_bracket() {
		let query = "(c2s,s2c)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![vec![
				Rule::Direction(RuleCommandDirection::C2S),
				Rule::Direction(RuleCommandDirection::S2C)
			]]
		)
	}

	#[test]
	fn should_parse_groups() {
		let query = "(c2s,s2c)(c2s)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![
				vec![Rule::Direction(RuleCommandDirection::C2S), Rule::Direction(RuleCommandDirection::S2C)],
				vec![Rule::Direction(RuleCommandDirection::C2S)]
			]
		)
	}

	#[test]
	fn should_parse_user() {
		let query = "(user=55)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(result, vec![vec![Rule::User(55)]])
	}

	#[test]
	fn should_parse_field() {
		let query = "(field=55)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(result, vec![vec![Rule::Field(55)]])
	}

	#[test]
	fn should_parse_template() {
		let query = "(template=155)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(result, vec![vec![Rule::Template(155)]])
	}

	#[test]
	fn should_parse_not_symbol() {
		let query = "(!c2s,!template=55,user!=100)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![vec![
				Rule::Not(Box::new(Rule::Direction(RuleCommandDirection::C2S))),
				Rule::Not(Box::new(Rule::Template(55))),
				Rule::Not(Box::new(Rule::User(100))),
			]]
		)
	}

	#[test]
	fn should_ignore_space() {
		let query = "( user = 55 , template   =    100)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(result, vec![vec![Rule::User(55), Rule::Template(100)]])
	}

	#[test]
	fn should_alias() {
		let query = "(u=55,t=100)";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(result, vec![vec![Rule::User(55), Rule::Template(100)]])
	}

	#[test]
	fn should_root_object_id() {
		let query = "(object=root(100),o!=root(55))";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![vec![
				Rule::Object(GameObjectId::new(100, ObjectOwner::Root)),
				Rule::Not(Box::new(Rule::Object(GameObjectId::new(55, ObjectOwner::Root))))
			]]
		)
	}

	#[test]
	fn should_user_object_id() {
		let query = "(object=user(5, 100),object!=user(5 ,100))";
		let result = parser().parse(query.as_ref()).unwrap();
		assert_eq!(
			result,
			vec![vec![
				Rule::Object(GameObjectId::new(100, ObjectOwner::User(5))),
				Rule::Not(Box::new(Rule::Object(GameObjectId::new(100, ObjectOwner::User(5)))))
			]]
		);
	}

	#[test]
	fn should_fail_when_wrong_filter() {
		let query = "(tttt=555)";
		match parser().parse(query.as_ref()) {
			Ok(_) => {
				assert!(false)
			}
			Err(_) => {
				assert!(true)
			}
		}
	}
}
