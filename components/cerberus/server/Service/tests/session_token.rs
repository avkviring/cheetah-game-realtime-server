use games_cheetah_cerberus_library::cerberus::{JWTTokenParser, JWTTokenParserError};
use games_cheetah_cerberus_service::token::*;
use std::fmt::format;
use std::thread;
use std::time::Duration;

pub mod helper;

#[test]
fn session_token_should_correct() {
    let service = JWTTokensService {
        session_exp_in_sec: 1,
        refresh_exp_in_sec: 1,
        private_key: helper::PRIVATE_KEY.to_string(),
        public_key: helper::PUBLIC_KEY.to_string(),
    };
    let user_id = "some-user-id".to_string();
    let tokens = service.create_tokens(user_id.clone(), "some-device-id".to_string());

    let parser = JWTTokenParser::new(helper::PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);
    assert!(matches!(user_id_from_token, Result::Ok(value) if value==user_id))
}

#[test]
fn session_token_should_exp() {
    let service = JWTTokensService {
        session_exp_in_sec: 1,
        refresh_exp_in_sec: 0,
        private_key: helper::PRIVATE_KEY.to_string(),
        public_key: helper::PUBLIC_KEY.to_string(),
    };
    let user_id = "some-user-id".to_string();
    let tokens = service.create_tokens(user_id.clone(), "some-device-id".to_string());
    thread::sleep(Duration::from_secs(2));
    let parser = JWTTokenParser::new(helper::PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);
    assert!(matches!(
        user_id_from_token,
        Result::Err(JWTTokenParserError::Expired)
    ))
}

#[test]
fn session_token_should_fail_if_not_correct() {
    let service = JWTTokensService {
        session_exp_in_sec: 1,
        refresh_exp_in_sec: 0,
        private_key: helper::PRIVATE_KEY.to_string(),
        public_key: helper::PUBLIC_KEY.to_string(),
    };
    let user_id = "some-user-id".to_string();
    let tokens = service.create_tokens(user_id.clone(), "some-device-id".to_string());
    let parser = JWTTokenParser::new(helper::PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session.replace("WQifQ", "WqifQ"));
    assert!(matches!(
        user_id_from_token,
        Result::Err(JWTTokenParserError::InvalidSignature)
    ))
}
