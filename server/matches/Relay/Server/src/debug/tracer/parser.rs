use std::cmp::min;

use crate::debug::tracer::filter::{Rule, RuleCommandDirection};

#[derive(Debug)]
pub enum ParseError {
	UnknownField(String),
	InternalError,
	ValueFormatError(String),
	UnknownOperation(String),
	RightBracketNotFound(String),
}
#[derive(Debug, Eq, PartialEq)]
enum Token {
	And,
	Or,
	Rule(Rule),
}
#[derive(Debug)]
enum Op {
	Equal,
	NotEqual,
}

///
/// Парсер запросов вида name=value, с поддержкой &&, ||,()
///
pub fn parse(query: &str) -> Result<Rule, ParseError> {
	let query = query.replace(" ", "");
	if query.is_empty() {
		Result::Ok(Rule::True)
	} else {
		match parse_with_bracket(query)? {
			Token::And => Result::Err(ParseError::InternalError),
			Token::Or => Result::Err(ParseError::InternalError),
			Token::Rule(rule) => Result::Ok(rule),
		}
	}
}

fn parse_with_bracket(mut query: String) -> Result<Token, ParseError> {
	let mut tokens = vec![];
	while !query.is_empty() {
		match query.chars().next().unwrap() {
			'(' => {
				let right_bracket = find_right_bracket(&query)?;
				let expression = (&query[1..right_bracket]).to_string();
				let token = parse_with_bracket(expression)?;
				tokens.push(token);
				query = (&query[right_bracket..]).to_string();
			}
			')' => {
				query = (&query[1..]).to_string();
			}
			'&' => {
				query = (&query[2..]).to_string();
				tokens.push(Token::And);
			}
			'|' => {
				query = (&query[2..]).to_string();
				tokens.push(Token::Or);
			}
			_ => {
				let (token, stripped_query) = parse_field(query.to_string())?;
				tokens.push(token);
				query = stripped_query;
			}
		}
	}
	match reduce(tokens)? {
		Token::And => Result::Err(ParseError::InternalError),
		Token::Or => Result::Err(ParseError::InternalError),
		Token::Rule(rule) => Result::Ok(Token::Rule(rule)),
	}
}

fn find_right_bracket(query: &String) -> Result<usize, ParseError> {
	let mut deep = 0;
	for (index, x) in query.chars().into_iter().enumerate() {
		if x == '(' {
			deep += 1;
		};
		if x == ')' {
			deep -= 1;
		};
		if deep == 0 {
			return Result::Ok(index);
		}
	}
	Result::Err(ParseError::RightBracketNotFound(query.to_owned()))
}

///
/// Преобразовать Token::And, Token::Or в Token::Rule с учетом приоритетов
/// В итоге из набора токенов должен остаться только один
///
fn reduce(mut source_tokens: Vec<Token>) -> Result<Token, ParseError> {
	let mut dest_tokens = vec![];
	for prior in [Token::And, Token::Or] {
		while !source_tokens.is_empty() {
			let token = source_tokens.remove(0);
			if let Token::Rule(_) = &token {
				dest_tokens.push(token);
			} else if token == prior {
				let result = reduce_token(&mut source_tokens, &mut dest_tokens, &token);
				dest_tokens.push(result?);
			} else {
				dest_tokens.push(token);
			}
		}
		source_tokens = dest_tokens;
		dest_tokens = vec![];
	}
	Result::Ok(source_tokens.remove(0))
}

///
/// Преобразовать один токен Token::And или Token::Or в Token::Rule
/// token_rule_1 token_and token_rule_2 token_and token_rule_3
/// преобразуется в Rule:And(rule_1,rule_2, rule_3)
///

fn reduce_token(source_tokens: &mut Vec<Token>, dest_tokens: &mut Vec<Token>, token: &Token) -> Result<Token, ParseError> {
	let left = get_rule(dest_tokens.remove(dest_tokens.len() - 1))?;
	let right = get_rule(source_tokens.remove(0))?;
	let rules = vec![left, right].into_iter().flat_map(|r| token.expand(r)).collect();
	Result::Ok(Token::Rule(token.create_rule(rules)))
}

fn get_rule(token: Token) -> Result<Rule, ParseError> {
	match token {
		Token::And => Result::Err(ParseError::InternalError),
		Token::Or => Result::Err(ParseError::InternalError),
		Token::Rule(rule) => Result::Ok(rule),
	}
}

fn parse_field(query: String) -> Result<(Token, String), ParseError> {
	if let Some(stripped) = query.strip_prefix("c2s") {
		Result::Ok((
			Token::Rule(Rule::Direction(RuleCommandDirection::C2S)),
			stripped.to_ascii_lowercase(),
		))
	} else if let Some(stripped) = query.strip_prefix("s2c") {
		Result::Ok((
			Token::Rule(Rule::Direction(RuleCommandDirection::S2C)),
			stripped.to_ascii_lowercase(),
		))
	} else {
		let (field, op, query) = get_field(query)?;
		let (value, query) = get_value(query);
		let result = match field.as_str() {
			"user" => {
				let id = to_id(value)?;
				Result::Ok(Rule::User(id as u16))
			}
			"template" => {
				let id = to_id(value)?;
				Result::Ok(Rule::Template(id as u16))
			}
			"field" => {
				let id = to_id(value)?;
				Result::Ok(Rule::Field(id as u16))
			}
			"id" => {
				let id = to_id(value)?;
				Result::Ok(Rule::ObjectId(id as u32))
			}
			"owner" => {
				if value == "room" {
					Result::Ok(Rule::RoomOwner)
				} else {
					let id = to_id(value)?;
					Result::Ok(Rule::UserOwner(id as u16))
				}
			}
			_ => return Err(ParseError::UnknownField(field)),
		};
		result.map(|rule| {
			(
				Token::Rule(match op {
					Op::Equal => rule,
					Op::NotEqual => Rule::Not(Box::new(rule)),
				}),
				query,
			)
		})
	}
}

fn to_id(value: String) -> Result<u64, ParseError> {
	value.parse().map_err(|_| ParseError::ValueFormatError(value))
}

///
/// Получить имя поля, а также операцию и урезанную исходную строку
///
fn get_field(query: String) -> Result<(String, Op, String), ParseError> {
	let eq_index = query.find("=");
	let not_index = query.find("!=");
	if eq_index.is_none() && not_index.is_none() {
		Result::Err(ParseError::UnknownOperation(query))
	} else {
		let eq_index = eq_index.unwrap_or(usize::MAX);
		let not_index = not_index.unwrap_or(usize::MAX);
		let (end_item_index, op, size) = if eq_index < not_index {
			(eq_index, Op::Equal, 1)
		} else {
			(not_index, Op::NotEqual, 2)
		};
		let field = &query[0..end_item_index];
		let query = query[end_item_index + size..].to_string();
		Result::Ok((field.to_string(), op, query))
	}
}

///
/// Получить значение поле и урезанную исходную строку
///
fn get_value(query: String) -> (String, String) {
	let and_op = query.find("&&").unwrap_or_else(|| query.len());
	let or_op = query.find("||").unwrap_or_else(|| query.len());
	let position = min(and_op, or_op);
	(query[..position].to_string(), query[position..].to_string())
}

impl Token {
	///
	/// Функция для объединения одно
	///
	pub fn create_rule(&self, rules: Vec<Rule>) -> Rule {
		match self {
			Token::And => Rule::AndRule(rules),
			Token::Or => Rule::OrRule(rules),
			Token::Rule(_) => {
				panic!("unsupported")
			}
		}
	}

	///
	/// Раскрыть rule как набор внутренних rule
	/// необходимо для преобразования правил And(rule1,And(rule2,rule3) в And(rule1, rule2, rule3)
	///
	pub fn expand(&self, rule: Rule) -> Vec<Rule> {
		match self {
			Token::And => {
				if let Rule::AndRule(rules) = rule {
					rules
				} else {
					vec![rule]
				}
			}
			Token::Or => {
				if let Rule::OrRule(rules) = rule {
					rules
				} else {
					vec![rule]
				}
			}
			Token::Rule(_) => {
				panic!("unsupported")
			}
		}
	}
}

#[cfg(test)]
mod test {
	use crate::debug::tracer::filter::{Rule, RuleCommandDirection};
	use crate::debug::tracer::parser::{parse, ParseError};

	#[test]
	fn should_parse_empty() {
		let query = "";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::True)
	}

	#[test]
	fn should_parse_user() {
		let query = "user=55";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::User(55))
	}

	#[test]
	fn should_parse_field() {
		let query = "field=55";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::Field(55))
	}

	#[test]
	fn should_parse_template() {
		let query = "template=155";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::Template(155))
	}
	#[test]
	fn should_parse_object_id() {
		let query = "id=155";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::ObjectId(155))
	}
	#[test]
	fn should_parse_room_owner() {
		let query = "owner=room";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::RoomOwner)
	}

	#[test]
	fn should_parse_user_owner() {
		let query = "owner=55";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::UserOwner(55))
	}

	#[test]
	fn should_parse_c2s() {
		let query = "c2s";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::Direction(RuleCommandDirection::C2S))
	}
	#[test]
	fn should_parse_s2c() {
		let query = "s2c";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::Direction(RuleCommandDirection::S2C))
	}

	#[test]
	fn should_parse_not() {
		let query = "owner!=55";
		let result = parse(query).unwrap();
		assert_eq!(result, Rule::Not(Box::new(Rule::UserOwner(55))))
	}

	#[test]
	fn should_parse_or() {
		let query = "c2s || user=55";
		let result = parse(query).unwrap();
		assert_eq!(
			result,
			Rule::OrRule(vec![Rule::Direction(RuleCommandDirection::C2S), Rule::User(55)])
		)
	}
	#[test]
	fn should_parse_more_two_or() {
		let query = "c2s || user=55 || template=10";
		let result = parse(query).unwrap();
		assert_eq!(
			result,
			Rule::OrRule(vec![
				Rule::Direction(RuleCommandDirection::C2S),
				Rule::User(55),
				Rule::Template(10)
			])
		)
	}

	#[test]
	fn should_parse_and() {
		let query = "user=55 && c2s";
		let result = parse(query).unwrap();
		assert_eq!(
			result,
			Rule::AndRule(vec![Rule::User(55), Rule::Direction(RuleCommandDirection::C2S)])
		)
	}
	#[test]
	fn should_parse_more_two_and() {
		let query = "user=55 && c2s && field=100";
		let result = parse(query).unwrap();
		assert_eq!(
			result,
			Rule::AndRule(vec![
				Rule::User(55),
				Rule::Direction(RuleCommandDirection::C2S),
				Rule::Field(100)
			])
		)
	}

	#[test]
	fn should_parse_with_bracket() {
		let query = "user=55 && field=10 && (template=20 || template=30) && owner=room";
		let result = parse(query).unwrap();
		assert_eq!(
			result,
			Rule::AndRule(vec![
				Rule::User(55),
				Rule::Field(10),
				Rule::OrRule(vec![Rule::Template(20), Rule::Template(30)]),
				Rule::RoomOwner
			])
		)
	}

	#[test]
	fn should_fail_when_wrong_field() {
		let query = "wrong=555";
		assert!(matches!(parse(query), Result::Err(ParseError::UnknownField(value)) if 
			value=="wrong"));
	}
	#[test]
	fn should_fail_when_wrong_value() {
		let query = "id=ttt";
		assert!(matches!(parse(query), Result::Err(ParseError::ValueFormatError(value)) if 
			value=="ttt"));
	}

	#[test]
	fn should_fail_when_wrong_bracket() {
		let query = "(id=ttt";
		assert!(
			matches!(parse(query), Result::Err(ParseError::RightBracketNotFound(value)) if 
			value=="(id=ttt")
		);
	}

	#[test]
	fn should_fail_when_wrong_operation() {
		let query = "id=1 & template=5";
		assert!(matches!(parse(query), Result::Err(ParseError::ValueFormatError(value)) if 
			value=="1&template=5"));
	}
}
