use std::sync::mpsc::Sender;
use std::time::Duration;

use futures::executor::block_on;
use testcontainers::clients::Cli;
use testcontainers::{images, Docker};
use tonic::Response;

use games_cheetah_cerberus_library::JWTTokenParser;
use games_cheetah_cerberus_service::proto::*;
use games_cheetah_cerberus_service::server::*;

pub mod helper;

#[tokio::main]
pub async fn start_server(tx: Sender<()>) {
    let cli = Cli::default();
    let node = cli.run(images::redis::Redis::default());
    let port = node.get_host_port(6379).unwrap();
    tx.send(());
    run_grpc_server(
        helper::PUBLIC_KEY.to_owned(),
        helper::PRIVATE_KEY.to_owned(),
        "localhost".to_owned(),
        port,
    )
    .await;
}

#[tokio::test]
pub async fn test() {
    let (tx, rx) = std::sync::mpsc::channel();
    let handler = std::thread::spawn(|| {
        start_server(tx);
    });
    rx.recv().unwrap();
    std::thread::sleep(Duration::from_millis(1000));

    let user_id = "some-user-id".to_owned();
    let device_id = "iphone se".to_owned();

    // проверяем создание токена
    let mut internal_client =
        internal::cerberus_client::CerberusClient::connect("http://127.0.0.1:5001")
            .await
            .unwrap();
    let request = tonic::Request::new(internal::CreateTokenRequest {
        user_id: user_id.clone(),
        device_id: device_id.clone(),
    });
    let result: Response<types::TokensReply> = internal_client.create(request).await.unwrap();
    let tokens = result.into_inner();
    let parser = JWTTokenParser::new(helper::PUBLIC_KEY.to_owned());
    assert!(
        matches!(parser.get_user_id(tokens.session.to_owned()), Result::Ok(value) if value==user_id)
    );

    // проверяем обновление токена
    let mut external_client =
        external::cerberus_client::CerberusClient::connect("http://127.0.0.1:5002")
            .await
            .unwrap();

    let request = tonic::Request::new(external::RefreshTokenRequest {
        token: tokens.refresh,
    });
    let result: Response<types::TokensReply> = external_client.refresh(request).await.unwrap();
    let tokens = result.into_inner();
    let parser = JWTTokenParser::new(helper::PUBLIC_KEY.to_owned());
    assert!(
        matches!(parser.get_user_id(tokens.session.to_owned()), Result::Ok(value) if value==user_id)
    );
}
