use cheetah_auth_cerberus::test_helper;
use cheetah_microservice::jwt::JWTTokenParser;
use cheetah_microservice::proto::auth::cerberus::types::Tokens;
use cheetah_microservice::proto::auth::cookie::external as cookie;
use cheetah_microservice::proto::auth::google::external as google;
use cheetah_microservice::tonic::{self, metadata::MetadataValue, Request};
use jsonwebtoken_google::test_helper::TokenClaims;
use std::collections::HashMap;
use std::time::Duration;
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use testcontainers::images::redis::Redis;
use testcontainers::{images, Container, Docker};
use tokio::task::JoinHandle;

// async fn setup_postgresql_storage(cli: &Cli) -> (sqlx::PgPool, Container<'_, Cli, Postgres>) {
//     let mut env = HashMap::default();
//     env.insert("POSTGRES_USER".to_owned(), "authentication".to_owned());
//     env.insert("POSTGRES_PASSWORD".to_owned(), "passwd".to_owned());
//     let image = images::postgres::Postgres::default()
//         .with_version(13)
//         .with_env_vars(env);
//     let node = cli.run(image);
//     let port = node.get_host_port(5432).unwrap();
//     let pool = cheetah_auth_cookie::storage::create_postgres_pool(
//         "authentication",
//         "authentication",
//         "passwd",
//         "127.0.0.1",
//         port,
//     )
//     .await;
//
//     sqlx::query(
//         r"
// create table if not exists users (
//     id          bigserial not null constraint users_pk primary key,
//     ip          inet not null,
//     create_time timestamp default CURRENT_TIMESTAMP not null
// )",
//     )
//     .execute(&pool)
//     .await
//     .unwrap();
//
//     sqlx::query(
//         r"
// create table cookie_users (
// 	user_id bigint not null primary key,
// 	cookie char(128) not null,
// 	linked bool default false
// )",
//     )
//     .execute(&pool)
//     .await
//     .unwrap();
//
//     cheetah_auth_google::storage::migrate_db(&pool).await;
//
//     (pool, node)
// }
//
// pub async fn setup(
//     cli: &Cli,
//     internal_cerberus_port: u16,
//     external_cerberus_port: u16,
//     user_port: u16,
//     service_port: u16,
//     public_google_key_url: String,
//     public_jwt_key: String,
// ) -> (
//     Container<'_, Cli, Redis>,
//     Container<'_, Cli, Postgres>,
//     JoinHandle<()>,
//     JoinHandle<()>,
//     JoinHandle<()>,
// ) {
//     let (handler_cerberus, redis_container) =
//         test_helper::stub_cerberus_grpc_server(internal_cerberus_port, external_cerberus_port)
//             .await;
//
//     let (pool, postgres_container) = setup_postgresql_storage(cli).await;
//
//     let user_pool = pool.clone();
//     let handler_user = tokio::spawn(async move {
//         cheetah_auth_user::run_grpc_server(user_pool, user_port).await;
//     });
//
//     let handler_google = tokio::spawn(async move {
//         cheetah_auth_google::run_grpc_server(
//             pool,
//             service_port,
//             &format!("http://127.0.0.1:{}", internal_cerberus_port),
//             &format!("http://127.0.0.1:{}", user_port),
//             public_jwt_key,
//             jsonwebtoken_google::Parser::new_with_custom_cert_url(
//                 jsonwebtoken_google::test_helper::CLIENT_ID,
//                 &public_google_key_url,
//             ),
//         )
//         .await;
//     });
//
//     tokio::time::sleep(Duration::from_secs(2)).await;
//     (
//         redis_container,
//         postgres_container,
//         handler_cerberus,
//         handler_user,
//         handler_google,
//     )
// }
//
// #[tokio::test]
// pub async fn should_register_and_login() {
//     let cli = Cli::default();
//     let service_port = 6004;
//
//     let (token, public_key_server) = jsonwebtoken_google::test_helper::setup_public_key_server(
//         &TokenClaims::new_with_expire(Duration::from_secs(100)),
//     );
//
//     let _handlers = setup(
//         &cli,
//         6000,
//         6001,
//         6002,
//         6003,
//         service_port,
//         public_key_server.url("/"),
//         test_helper::PUBLIC_KEY.to_string(),
//     )
//     .await;
//
//     let service_url = format!("http://127.0.0.1:{}", service_port);
//     let mut client = google::google_client::GoogleClient::connect(service_url)
//         .await
//         .unwrap();
//
//     let result: tonic::Response<google::RegisterOrLoginResponse> = client
//         .register_or_login(tonic::Request::new(google::RegisterOrLoginRequest {
//             google_token: token.clone(),
//             device_id: "some-device-id".to_owned(),
//         }))
//         .await
//         .unwrap();
//     let register_result = result.into_inner();
//
//     let result: tonic::Response<google::RegisterOrLoginResponse> = client
//         .register_or_login(tonic::Request::new(google::RegisterOrLoginRequest {
//             google_token: token,
//             device_id: "some-device-id".to_owned(),
//         }))
//         .await
//         .unwrap();
//     let login_result = result.into_inner();
//
//     let token_parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());
//
//     assert!(register_result.registered_player);
//     assert!(!login_result.registered_player);
//     assert_eq!(
//         token_parser
//             .get_player_id(register_result.tokens.unwrap().session)
//             .unwrap(),
//         token_parser
//             .get_player_id(login_result.tokens.unwrap().session)
//             .unwrap(),
//     );
// }
//
// #[tokio::test]
// pub async fn should_attach() {
//     let cli = Cli::default();
//     let cookie_port = 6103;
//     let service_port = 6104;
//
//     let (google_token, public_key_server) =
//         jsonwebtoken_google::test_helper::setup_public_key_server(&TokenClaims::new_with_expire(
//             Duration::from_secs(100),
//         ));
//     let _handlers = setup(
//         &cli,
//         6100,
//         6101,
//         6102,
//         cookie_port,
//         service_port,
//         public_key_server.url("/"),
//         test_helper::PUBLIC_KEY.to_string(),
//     )
//     .await;
//
//     let service_url = format!("http://127.0.0.1:{}", service_port);
//     let cookie_url = format!("http://127.0.0.1:{}", cookie_port);
//
//     // регистрируемся через cookie
//     let token_from_cookie = register_player_by_cookie(cookie_url).await;
//
//     // связываем игрока с google
//     let mut google_client = google::google_client::GoogleClient::connect(service_url)
//         .await
//         .unwrap();
//     let mut request = tonic::Request::new(google::AttachRequest {
//         google_token: google_token.clone(),
//         device_id: "some-device-id".to_owned(),
//     });
//     request.metadata_mut().insert(
//         "authorization",
//         MetadataValue::from_str(&format!("Bearer {}", token_from_cookie.session)).unwrap(),
//     );
//     google_client.attach(request).await.unwrap();
//
//     // входим через google
//     let request = tonic::Request::new(google::RegisterOrLoginRequest {
//         google_token,
//         device_id: "some-device-id".to_owned(),
//     });
//     let result: tonic::Response<google::RegisterOrLoginResponse> =
//         google_client.register_or_login(request).await.unwrap();
//
//     let google_tokens = result.into_inner().tokens.unwrap();
//
//     let token_parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());
//     // идентификаторы пользователей должны совпадать
//     assert_eq!(
//         token_parser
//             .get_player_id(token_from_cookie.session)
//             .unwrap(),
//         token_parser.get_player_id(google_tokens.session).unwrap()
//     );
// }
