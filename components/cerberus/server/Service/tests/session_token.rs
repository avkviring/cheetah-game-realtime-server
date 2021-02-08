use std::thread;
use std::time::Duration;

use games_cheetah_cerberus_library::{JWTTokenParser, SessionTokenError};
use helper::{stub_token_service, PUBLIC_KEY};

pub mod helper;

#[tokio::test]
async fn session_token_should_correct() {
    let (_node, service) = stub_token_service(1, 1);
    let user_id = "some-user-id".to_string();
    let tokens = service
        .create(user_id.clone(), "some-device-id".to_string())
        .await
        .unwrap();

    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);

    assert!(matches!(user_id_from_token, Result::Ok(value) if value==user_id))
}

#[tokio::test]
async fn session_token_should_exp() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens = service
        .create("some_user_id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    thread::sleep(Duration::from_secs(2));
    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session);
    assert!(matches!(
        user_id_from_token,
        Result::Err(SessionTokenError::Expired)
    ))
}

#[tokio::test]
async fn session_token_should_fail_if_not_correct() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    let parser = JWTTokenParser::new(PUBLIC_KEY.to_owned());
    let user_id_from_token = parser.get_user_id(tokens.session.replace("WQifQ", "WqifQ"));
    assert!(matches!(
        user_id_from_token,
        Result::Err(SessionTokenError::InvalidSignature)
    ))
}
