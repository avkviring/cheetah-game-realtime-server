use std::time::Duration;

use jsonwebtoken_google::test_helper::TokenClaims;
use testcontainers::clients::Cli;
use tonic::metadata::MetadataValue;
use tonic::Request;

use games_cheetah_authentication_service::proto::auth::external::cookie;
use games_cheetah_authentication_service::proto::auth::external::google;
use games_cheetah_authentication_service::proto::cerberus::types::Tokens;
use games_cheetah_cerberus_library::token::JWTTokenParser;
use games_cheetah_cerberus_service::test_helper;

pub mod helper;

#[tokio::test]
pub async fn should_register_and_login() {
    let cli = Cli::default();
    let service_port = 6004;

    let (token, public_key_server) = jsonwebtoken_google::test_helper::setup_public_key_server(
        &TokenClaims::new_with_expire(Duration::from_secs(100)),
    );

    let (_container_redis, _container_postgresql, _cerberus_handler, _auth_handler) =
        helper::setup(
            &cli,
            6000,
            6001,
            service_port,
            public_key_server.url("/"),
            test_helper::PUBLIC_KEY.to_string(),
        )
        .await;

    let service_url = format!("http://127.0.0.1:{}", service_port);
    let mut client = google::google_client::GoogleClient::connect(service_url)
        .await
        .unwrap();

    let result: tonic::Response<google::RegisterOrLoginResponse> = client
        .register_or_login(tonic::Request::new(google::RegisterOrLoginRequest {
            google_token: token.clone(),
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let register_result = result.into_inner();

    let result: tonic::Response<google::RegisterOrLoginResponse> = client
        .register_or_login(tonic::Request::new(google::RegisterOrLoginRequest {
            google_token: token,
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let login_result = result.into_inner();

    let token_parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());

    assert!(register_result.registered_player);
    assert!(!login_result.registered_player);
    assert_eq!(
        token_parser
            .get_player_id(register_result.tokens.unwrap().session)
            .unwrap(),
        token_parser
            .get_player_id(login_result.tokens.unwrap().session)
            .unwrap(),
    );
}

#[tokio::test]
pub async fn should_attach() {
    let cli = Cli::default();
    let service_port = 6104;

    let (google_token, public_key_server) =
        jsonwebtoken_google::test_helper::setup_public_key_server(&TokenClaims::new_with_expire(
            Duration::from_secs(100),
        ));
    let (_container_redis, _container_postgresql, _cerberus_handler, _auth_handler) =
        helper::setup(
            &cli,
            6100,
            6101,
            service_port,
            public_key_server.url("/"),
            test_helper::PUBLIC_KEY.to_string(),
        )
        .await;

    let service_url = format!("http://127.0.0.1:{}", service_port);

    // регистрируемся через cookie
    let token_from_cookie = register_player_by_cookie(service_url.clone()).await;

    // связываем игрока с google
    let mut google_client = google::google_client::GoogleClient::connect(service_url)
        .await
        .unwrap();
    let mut request = tonic::Request::new(google::AttachRequest {
        google_token: google_token.clone(),
        device_id: "some-device-id".to_owned(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(format!("Bearer {}", token_from_cookie.session).as_str()).unwrap(),
    );
    google_client.attach(request).await.unwrap();

    // входим через google
    let request = tonic::Request::new(google::RegisterOrLoginRequest {
        google_token,
        device_id: "some-device-id".to_owned(),
    });
    let result: tonic::Response<google::RegisterOrLoginResponse> =
        google_client.register_or_login(request).await.unwrap();

    let google_tokens = result.into_inner().tokens.unwrap();

    let token_parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());
    // идентификаторы пользователей должны совпадать
    assert_eq!(
        token_parser
            .get_player_id(token_from_cookie.session)
            .unwrap(),
        token_parser.get_player_id(google_tokens.session).unwrap()
    );
}

async fn register_player_by_cookie(service_url: String) -> Tokens {
    let mut cookie_client = cookie::cookie_client::CookieClient::connect(service_url)
        .await
        .unwrap();

    let registry_response: tonic::Response<cookie::RegistryResponse> = cookie_client
        .register(Request::new(cookie::RegistryRequest {
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let registry_response = registry_response.into_inner();
    registry_response.tokens.unwrap()
}
