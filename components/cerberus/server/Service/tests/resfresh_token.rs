use std::thread;
use std::time::Duration;

use testcontainers::*;

use games_cheetah_cerberus_library::{JWTTokenParser, SessionTokenError};
use games_cheetah_cerberus_service::token::*;
use helper::{stub_token_service, PUBLIC_KEY};

pub mod helper;

#[tokio::test]
async fn should_refresh_token_different_for_users() {
    let (_node, service) = stub_token_service(1, 100);
    let tokens_for_user_a = service
        .create("some-usera-id".to_owned(), "some-devicea-id".to_string())
        .await
        .unwrap();
    let tokens_for_user_b = service
        .create("some-userb-id".to_owned(), "some-deviceb-id".to_string())
        .await
        .unwrap();
    assert_ne!(tokens_for_user_a.refresh, tokens_for_user_b.refresh)
}

#[tokio::test]
async fn should_refresh_token() {
    let (_node, service) = stub_token_service(1, 100);

    let tokens = service
        .create("some-user-id".to_owned(), "some-device-id".to_owned())
        .await
        .unwrap();

    let new_tokens = service.refresh(tokens.refresh.clone()).await.unwrap();
    // проверяем что это действительно новые токены
    assert_ne!(tokens.session, new_tokens.session);
    assert_ne!(tokens.refresh, new_tokens.refresh);
    // проверяем работоспособность новых токенов
    let get_user_id_result =
        JWTTokenParser::new(helper::PUBLIC_KEY.to_owned()).get_user_id(new_tokens.session);
    assert!(matches!(get_user_id_result, Result::Ok(user_id) if user_id=="some-user-id"));

    // проверяем что новый refresh токен валидный
    service.refresh(new_tokens.refresh.clone()).await.unwrap();
}

///
/// Проверяем время жизни refresh токена
///
#[tokio::test]
async fn should_refresh_token_exp() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    thread::sleep(Duration::from_secs(2));
    let result = service.refresh(tokens.refresh).await;
    assert!(matches!(
        result,
        Result::Err(JWTTokensServiceError::Expired)
    ));
}

///
/// Проверяем реакцию на нарушение подписи
///
#[tokio::test]
async fn should_refresh_token_fail() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    assert!(matches!(
        service
            .refresh(tokens.refresh.replace("eyJleHA", "eyJleHB"))
            .await,
        Result::Err(JWTTokensServiceError::InvalidSignature)
    ));
}

///
/// Проверяем что refresh токен может быть использован один раз
///
#[tokio::test]
async fn should_refresh_token_can_use_once() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    service.refresh(tokens.refresh.clone()).await.unwrap();
    assert!(matches!(
        service.refresh(tokens.refresh).await,
        Result::Err(JWTTokensServiceError::InvalidId)
    ));
}

///
/// Проверяем что выдача нового refresh токена инвалидирует старые
///
#[tokio::test]
async fn should_refresh_token_can_invalidate_tokens() {
    let (_node, service) = stub_token_service(1, 1);
    let tokens_a = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    let tokens_b = service
        .create("some-user-id".to_owned(), "some-device-id".to_string())
        .await
        .unwrap();
    service.refresh(tokens_b.refresh.clone()).await.unwrap();
    assert!(matches!(
        service.refresh(tokens_a.refresh).await,
        Result::Err(JWTTokensServiceError::InvalidId)
    ));
}
