use std::thread;
use std::time::Duration;

use games_cheetah_cerberus_library::token::JWTTokenParser;
use games_cheetah_cerberus_service::token::*;
use helper::stub_token_service;

pub mod helper;

#[tokio::test]
async fn should_refresh_token_different_for_players() {
    let (_node, service) = stub_token_service(1, 100);
    let tokens_for_player_a = service
        .create(123, "some-devicea-id".to_string())
        .await
        .unwrap();
    let tokens_for_player_b = service
        .create(124, "some-deviceb-id".to_string())
        .await
        .unwrap();
    assert_ne!(tokens_for_player_a.refresh, tokens_for_player_b.refresh)
}

#[tokio::test]
async fn should_refresh_token() {
    let (_node, service) = stub_token_service(1, 100);

    let tokens = service
        .create(123, "some-device-id".to_owned())
        .await
        .unwrap();

    let new_tokens = service.refresh(tokens.refresh.clone()).await.unwrap();
    // проверяем что это действительно новые токены
    assert_ne!(tokens.session, new_tokens.session);
    assert_ne!(tokens.refresh, new_tokens.refresh);
    // проверяем работоспособность новых токенов
    let get_player_id_result =
        JWTTokenParser::new(helper::PUBLIC_KEY.to_owned()).get_player_id(new_tokens.session);
    assert!(matches!(get_player_id_result, Result::Ok(player) if player==123));

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
        .create(123, "some-device-id".to_string())
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
        .create(123, "some-device-id".to_string())
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
        .create(123, "some-device-id".to_string())
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
        .create(123, "some-device-id".to_string())
        .await
        .unwrap();
    let tokens_b = service
        .create(123, "some-device-id".to_string())
        .await
        .unwrap();
    service.refresh(tokens_b.refresh.clone()).await.unwrap();
    assert!(matches!(
        service.refresh(tokens_a.refresh).await,
        Result::Err(JWTTokensServiceError::InvalidId)
    ));
}
