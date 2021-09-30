use pom::parser::Parser;
use pom::parser::*;

use crate::room::debug::tracer::filter::{Rule, RuleCommandDirection};

///
/// Парсер текстового фильтра в структуру Rule
/// todo - неплохо бы сделать понятные сообщения об ошибках
///
fn parser<'a>() -> Parser<'a, u8, Vec<Vec<Rule>>> {
	let list = list(seq(b"(") * rules_group() - seq(b")"), space());
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
	one_of(b"0123456789")
		.repeat(0..)
		.map(|v| std::str::from_utf8(&v).unwrap().parse().unwrap())
}

///
/// поле вида name=id
///
fn field<'a>(name: &'a str) -> Parser<'a, u8, u64> {
	space() * seq(name.as_bytes()) * space() * seq(b"=") * space() * id()
}
///
/// поле вида name!=id
///
fn not_field<'a>(name: &'a str) -> Parser<'a, u8, u64> {
	space() * seq(name.as_bytes()) * space() * seq(b"!=") * space() * id()
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
		| field("user").map(|id| Rule::User(id as u16))
		| field("template").map(|id| Rule::Template(id as u16))
		| not_field("user").map(|id| Rule::Not(Box::new(Rule::User(id as u16))))
		| not_field("template").map(|id| Rule::Not(Box::new(Rule::Template(id as u16))))
}

#[cfg(test)]
mod test {
	use crate::room::debug::tracer::parser::{parser, Rule, RuleCommandDirection};

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
}
