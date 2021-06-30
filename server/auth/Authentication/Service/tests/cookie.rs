use testcontainers::clients::Cli;
use tonic::Request;

use cheetah_auth_authentication::proto::auth::cookie::external::*;
use cheetah_auth_cerberus::test_helper;

use crate::helper::setup;

pub mod helper;

#[tokio::test]
pub async fn should_registry_and_login_by_cookie() {
    let cli = Cli::default();
    let service_port = 5202;
    let (_container_redis, _container_postgresql, _cerberus_handler, _auth_handler) = setup(
        &cli,
        5200,
        5201,
        service_port,
        jsonwebtoken_google::Parser::GOOGLE_CERT_URL.to_owned(),
        test_helper::PUBLIC_KEY.to_string(),
    )
    .await;

    let cookie_addr = format!("http://127.0.0.1:{}", service_port);
    let mut cookie_client: cookie_client::CookieClient<tonic::transport::Channel> =
        cookie_client::CookieClient::connect(cookie_addr)
            .await
            .unwrap();

    // регистрируем нового игрока
    let registry_response: tonic::Response<RegistryResponse> = cookie_client
        .register(Request::new(RegistryRequest {
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let registry_response = registry_response.into_inner();
    let token = registry_response.tokens.unwrap();

    //входим с использованием cookie
    let cookie = registry_response.cookie.to_owned();
    let login_response: tonic::Response<LoginResponse> = cookie_client
        .login(Request::new(LoginRequest {
            cookie,
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let login_response = login_response.into_inner();
    assert_eq!(login_response.status, login_response::Status::Ok as i32);

    // проверяем что новый пользователь и вошедший по cookie пользователь - один и тот же
    let token_parser = cheetah_microservice::jwt::JWTTokenParser::new(
        cheetah_auth_cerberus::test_helper::PUBLIC_KEY.to_owned(),
    );
    let registered_player = token_parser
        .get_player_id(token.session.to_owned())
        .unwrap();
    let logged_player = token_parser
        .get_player_id(login_response.tokens.unwrap().session)
        .unwrap();

    assert_eq!(registered_player, logged_player);
}

#[tokio::test]
pub async fn should_not_login_by_wrong_cookie() {
    let cli = Cli::default();
    let service_port = 5102;
    let (_container_redis, _container_postgresql, _cerberus_handler, _auth_handler) = setup(
        &cli,
        5100,
        5101,
        service_port,
        jsonwebtoken_google::Parser::GOOGLE_CERT_URL.to_owned(),
        test_helper::PUBLIC_KEY.to_string(),
    )
    .await;

    let cookie_addr = format!("http://127.0.0.1:{}", service_port);
    let mut cookie_client: cookie_client::CookieClient<tonic::transport::Channel> =
        cookie_client::CookieClient::connect(cookie_addr)
            .await
            .unwrap();
    let login_response: tonic::Response<LoginResponse> = cookie_client
        .login(Request::new(LoginRequest {
            cookie: "some-wrong-cookie".to_owned(),
            device_id: "some-device-id".to_owned(),
        }))
        .await
        .unwrap();
    let login_response = login_response.into_inner();
    assert_eq!(
        login_response.status,
        login_response::Status::NotFound as i32
    );
    assert!(matches!(login_response.tokens, None));
}
