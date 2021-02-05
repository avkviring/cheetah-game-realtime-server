use std::thread;
use std::time::Duration;

use games_cheetah_cerberus_library::{JWTTokenParser, SessionTokenError};
use helper::{new_service_with_stub_storage, PUBLIC_KEY};

pub mod helper;

#[test]
fn session_token_should_correct() {
    let mut service = new_service_with_stub_storage(1, 1);
    let user_id = "some-user-id".to_string();
    let tokens = service.create_tokens(user_id.clone(), "some-device-id".to_string());

    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);
    assert!(matches!(user_id_from_token, Result::Ok(value) if value==user_id))
}

#[test]
fn session_token_should_exp() {
    let mut service = new_service_with_stub_storage(1, 0);
    let tokens = service.create_tokens("some_user_id".to_owned(), "some-device-id".to_string());
    thread::sleep(Duration::from_secs(2));
    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);
    assert!(matches!(
        user_id_from_token,
        Result::Err(SessionTokenError::Expired)
    ))
}

#[test]
fn session_token_should_fail_if_not_correct() {
    let mut service = new_service_with_stub_storage(1, 0);
    let tokens = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session.replace("WQifQ", "WqifQ"));
    assert!(matches!(
        user_id_from_token,
        Result::Err(SessionTokenError::InvalidSignature)
    ))
}
