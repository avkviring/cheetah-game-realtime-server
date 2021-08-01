use std::net::SocketAddr;
use std::time::Duration;

use jsonwebtoken_google::test_helper::TokenClaims;
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};
use tokio::task::JoinHandle;

use cheetah_auth_cerberus::test_helper;
use cheetah_auth_google::api::user::Id;
use cheetah_microservice::jwt::JWTTokenParser;
use cheetah_microservice::proto::auth::cerberus::types::Tokens;
use cheetah_microservice::proto::auth::cookie::external as cookie;
use cheetah_microservice::proto::auth::google::external as google;
use cheetah_microservice::tonic::transport::Uri;
use cheetah_microservice::tonic::{self, metadata::MetadataValue, Request};

pub async fn setup(
    cli: &Cli,
    internal_cerberus_service_port: u16,
    external_cerberus_service_port: u16,
    internal_user_service_port: u16,
    external_google_service_port: u16,
    public_google_key_url: String,
    public_jwt_key: String,
) -> (
    Container<'_, Cli, Redis>,
    Container<'_, Cli, Postgres>,
    JoinHandle<()>,
    JoinHandle<()>,
    JoinHandle<()>,
) {
    let (handler_cerberus, redis_container) = test_helper::stub_cerberus_grpc_server(
        internal_cerberus_service_port,
        external_cerberus_service_port,
    )
    .await;

    let (pools, container) =
        cheetah_microservice::test_helper::postgresql::create_psql_databases(&cli, 2).await;

    let user_pool = pools[0].clone();
    cheetah_auth_user::storage::migrate_db(&user_pool).await;
    let handler_user = tokio::spawn(async move {
        cheetah_auth_user::run_grpc_server(
            user_pool,
            format!("0.0.0.0:{}", internal_user_service_port)
                .parse()
                .unwrap(),
        )
        .await;
    });

    let google_pool = pools[1].clone();
    cheetah_auth_google::storage::migrate_db(&google_pool).await;
    let handler_google = tokio::spawn(async move {
        let cerberus_service_uri: Uri =
            format!("http://127.0.0.1:{}", internal_cerberus_service_port)
                .parse()
                .unwrap();
        let user_service_uri: Uri = format!("http://127.0.0.1:{}", internal_user_service_port)
            .parse()
            .unwrap();
        cheetah_auth_google::run_grpc_server(
            google_pool,
            external_google_service_port,
            cerberus_service_uri,
            user_service_uri,
            public_jwt_key,
            jsonwebtoken_google::Parser::new_with_custom_cert_url(
                jsonwebtoken_google::test_helper::CLIENT_ID,
                &public_google_key_url,
            ),
        )
        .await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;
    (
        redis_container,
        container,
        handler_cerberus,
        handler_user,
        handler_google,
    )
}

#[tokio::test]
pub async fn should_register_and_login() {
    let cli = Cli::default();
    let service_port = 6004u16;

    let (token, public_key_server) = jsonwebtoken_google::test_helper::setup_public_key_server(
        &TokenClaims::new_with_expire(Duration::from_secs(100)),
    );

    let _handlers = setup(
        &cli,
        6000,
        6001,
        6002,
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
    let cerberus_internal_service_port = 6100;
    let user_internal_service_port = 6103;
    let google_internal_service_port = 6104;

    let (google_token, public_key_server) =
        jsonwebtoken_google::test_helper::setup_public_key_server(&TokenClaims::new_with_expire(
            Duration::from_secs(100),
        ));
    let _handlers = setup(
        &cli,
        cerberus_internal_service_port,
        6101,
        user_internal_service_port,
        google_internal_service_port,
        public_key_server.url("/"),
        test_helper::PUBLIC_KEY.to_string(),
    )
    .await;

    let user_internal_service_uri: Uri = format!("http://127.0.0.1:{}", user_internal_service_port)
        .parse()
        .unwrap();
    let google_internal_service_uri = format!("http://127.0.0.1:{}", google_internal_service_port);
    let cerberus_internal_service_uri: Uri =
        format!("http://127.0.0.1:{}", cerberus_internal_service_port)
            .parse()
            .unwrap();

    // регистрируемся через cookie
    let token_from_cookie: Tokens =
        cheetah_auth_google::api::cerberus::Client::new(cerberus_internal_service_uri)
            .create_token("some-device", Id::from(100))
            .await
            .unwrap();

    // связываем игрока с google
    let mut google_client =
        google::google_client::GoogleClient::connect(google_internal_service_uri)
            .await
            .unwrap();
    let mut request = tonic::Request::new(google::AttachRequest {
        google_token: google_token.clone(),
        device_id: "some-device-id".to_owned(),
    });
    request.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", token_from_cookie.session)).unwrap(),
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
