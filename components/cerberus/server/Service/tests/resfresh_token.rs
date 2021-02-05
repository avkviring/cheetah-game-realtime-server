use std::thread;
use std::time::Duration;

use testcontainers::*;

use crate::helper::new_service_with_redis_storage;
use games_cheetah_cerberus_library::{JWTTokenParser, SessionTokenError};
use games_cheetah_cerberus_service::token::*;
use helper::{new_service_with_stub_storage, PUBLIC_KEY};

pub mod helper;

#[test]
fn should_refresh_token_different_for_users() {
    let mut service = new_service_with_stub_storage(1, 100);
    let tokens_for_user_a =
        service.create_tokens("some-usera-id".to_owned(), "some-devicea-id".to_string());
    let tokens_for_user_b =
        service.create_tokens("some-userb-id".to_owned(), "some-deviceb-id".to_string());

    assert_ne!(tokens_for_user_a.refresh, tokens_for_user_b.refresh)
}

#[test]
fn should_refresh_token() {
    let mut service = new_service_with_stub_storage(1, 100);
    let tokens = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    let new_tokens = service.refresh(tokens.refresh.clone()).unwrap();
    // проверяем что это действительно новые токены
    assert_ne!(tokens.session, new_tokens.session);
    assert_ne!(tokens.refresh, new_tokens.refresh);
    // проверяем работоспособность новых токенов
    let get_user_id_result =
        JWTTokenParser::new(helper::PUBLIC_KEY.to_owned()).get_user_id(new_tokens.session);
    assert!(matches!(get_user_id_result, Result::Ok(user_id) if user_id=="some-user-id"));

    // првоверяем что новый refresh токен валидный
    service.refresh(tokens.refresh.clone()).unwrap();
}

///
/// Проверяем время жизни refresh токена
///
#[test]
fn should_refresh_token_exp() {
    let mut service = new_service_with_stub_storage(1, 1);
    let tokens = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    thread::sleep(Duration::from_secs(2));
    assert!(matches!(
        service.refresh(tokens.refresh),
        Result::Err(RefreshTokenError::Expired)
    ));
}

///
/// Проверяем реакцию на нарушение подписи
///
#[test]
fn should_refresh_token_fail() {
    let mut service = new_service_with_stub_storage(1, 1);
    let tokens = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    assert!(matches!(
        service.refresh(tokens.refresh.replace("eyJleHA", "eyJleHB")),
        Result::Err(RefreshTokenError::InvalidSignature)
    ));
}

///
/// Проверяем что refresh токен может быть использован один раз
///
#[test]
fn should_refresh_token_can_use_once() {
    let cli = clients::Cli::default();
    let node = cli.run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    let mut service = new_service_with_redis_storage(1, 100, port);
    let tokens = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    service.refresh(tokens.refresh.clone()).unwrap();
    assert!(matches!(
        service.refresh(tokens.refresh),
        Result::Err(RefreshTokenError::InvalidId)
    ));
}

///
/// Проверяем что выдача нового refresh токена инвалидирует старые
///
#[test]
fn should_refresh_token_can_invalidate_tokens() {
    let cli = clients::Cli::default();
    let node = cli.run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    let mut service = new_service_with_redis_storage(1, 100, port);
    let tokens_a = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    let tokens_b = service.create_tokens("some-user-id".to_owned(), "some-device-id".to_string());
    service.refresh(tokens_b.refresh.clone()).unwrap();
    assert!(matches!(
        service.refresh(tokens_a.refresh),
        Result::Err(RefreshTokenError::InvalidId)
    ));
}
