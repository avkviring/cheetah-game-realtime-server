use cheetah_auth_cerberus::test_helper;
use cheetah_microservice::proto::auth::cookie::external::*;
use cheetah_microservice::tonic::{transport::Channel, Request, Response};
use std::collections::HashMap;
use std::time::Duration;
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};
use tokio::task::JoinHandle;

async fn setup_postgresql_storage(cli: &Cli) -> (sqlx::PgPool, Container<'_, Cli, Postgres>) {
    let mut env = HashMap::default();
    env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
    env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
    let image = images::postgres::Postgres::default()
        .with_version(13)
        .with_env_vars(env);
    let node = cli.run(image);
    let port = node.get_host_port(5432).unwrap();
    let pool = cheetah_auth_cookie::storage::create_postgres_pool(
        "authentication",
        "authentication",
        "passwd",
        "127.0.0.1",
        port,
    )
    .await;
    cheetah_auth_cookie::storage::migrate_db(&pool).await;

    (pool, node)
}

pub async fn setup(
    cli: &Cli,
    internal_cerberus_port: u16,
    external_cerberus_port: u16,
    user_port: u16,
    service_port: u16,
) -> (
    Container<'_, Cli, Redis>,
    Container<'_, Cli, Postgres>,
    JoinHandle<()>,
    JoinHandle<()>,
    JoinHandle<()>,
) {
    let (handler_cerberus, redis_container) =
        test_helper::stub_cerberus_grpc_server(internal_cerberus_port, external_cerberus_port)
            .await;

    let (pool, postgres_container) = setup_postgresql_storage(cli).await;

    let user_pool = pool.clone();
    cheetah_auth_user::storage::migrate_db(&pool).await;

    let handler_user = tokio::spawn(async move {
        cheetah_auth_user::run_grpc_server(user_pool, user_port).await;
    });

    let handler_cookie = tokio::spawn(async move {
        cheetah_auth_cookie::run_grpc_server(
            pool,
            &format!("http://127.0.0.1:{}", internal_cerberus_port),
            &format!("http://127.0.0.1:{}", user_port),
            service_port,
        )
        .await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;
    (
        redis_container,
        postgres_container,
        handler_cerberus,
        handler_user,
        handler_cookie,
    )
}

// #[tokio::test]
// pub async fn should_registry_and_login_by_cookie() {
//     let cli = Cli::default();
//     let service_port = 5203;
//     let (
//         _container_redis,
//         _container_postgresql,
//         _cerberus_handler,
//         _user_handler,
//         _cookie_handler,
//     ) = setup(&cli, 5200, 5201, 5202, service_port).await;
//
//     let cookie_addr = format!("http://127.0.0.1:{}", service_port);
//     let mut cookie_client: cookie_client::CookieClient<Channel> =
//         cookie_client::CookieClient::connect(cookie_addr)
//             .await
//             .unwrap();
//
//     // регистрируем нового игрока
//     let registry_response: Response<RegistryResponse> = cookie_client
//         .register(Request::new(RegistryRequest {
//             device_id: "some-device-id".to_string(),
//         }))
//         .await
//         .unwrap();
//     let registry_response = registry_response.into_inner();
//     let token = registry_response.tokens.unwrap();
//
//     //входим с использованием cookie
//     let cookie = registry_response.cookie.clone();
//     let login_response: Response<LoginResponse> = cookie_client
//         .login(Request::new(LoginRequest {
//             cookie,
//             device_id: "some-device-id".to_string(),
//         }))
//         .await
//         .unwrap();
//     let login_response = login_response.into_inner();
//     assert_eq!(login_response.status, login_response::Status::Ok as i32);
//
//     // проверяем что новый пользователь и вошедший по cookie пользователь - один и тот же
//     let token_parser = cheetah_microservice::jwt::JWTTokenParser::new(
//         cheetah_auth_cerberus::test_helper::PUBLIC_KEY.to_string(),
//     );
//     let registered_player = token_parser.get_player_id(token.session.clone()).unwrap();
//     let logged_player = token_parser
//         .get_player_id(login_response.tokens.unwrap().session)
//         .unwrap();
//
//     assert_eq!(registered_player, logged_player);
// }

// #[tokio::test]
// pub async fn should_not_login_by_wrong_cookie() {
//     let cli = Cli::default();
//     let service_port = 5103;
//     let (
//         _container_redis,
//         _container_postgresql,
//         _cerberus_handler,
//         _user_handler,
//         _cookie_handler,
//     ) = setup(&cli, 5100, 5101, 5102, service_port).await;
//
//     let cookie_addr = format!("http://127.0.0.1:{}", service_port);
//     let mut cookie_client: cookie_client::CookieClient<Channel> =
//         cookie_client::CookieClient::connect(cookie_addr)
//             .await
//             .unwrap();
//     let login_response: Response<LoginResponse> = cookie_client
//         .login(Request::new(LoginRequest {
//             cookie: "some-wrong-cookie".to_owned(),
//             device_id: "some-device-id".to_owned(),
//         }))
//         .await
//         .unwrap();
//     let login_response = login_response.into_inner();
//     assert_eq!(
//         login_response.status,
//         login_response::Status::NotFound as i32
//     );
//     assert!(matches!(login_response.tokens, None));
// }
