use tonic::Response;

use cheetah_auth_cerberus::proto::*;
use cheetah_microservice::jwt::JWTTokenParser;

#[path = "../src/test_helper/mod.rs"]
pub mod test_helper;

#[tokio::test]
pub async fn test_server() {
    let (_handler, _redis) = test_helper::stub_cerberus_grpc_server(7001, 7002).await;

    let player = 123;
    let device_id = "iphone se".to_owned();

    // проверяем создание токена
    let mut internal_client =
        internal::cerberus_client::CerberusClient::connect("http://127.0.0.1:7001")
            .await
            .unwrap();
    let request = tonic::Request::new(internal::CreateTokenRequest {
        player,
        device_id: device_id.clone(),
    });
    let result: Response<types::Tokens> = internal_client.create(request).await.unwrap();
    let tokens = result.into_inner();
    let parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());
    assert!(
        matches!(parser.get_player_id(tokens.session.to_owned()), Result::Ok(value) if value==player)
    );

    // проверяем обновление токена
    let mut external_client =
        external::cerberus_client::CerberusClient::connect("http://127.0.0.1:7002")
            .await
            .unwrap();

    let request = tonic::Request::new(external::RefreshTokenRequest {
        token: tokens.refresh,
    });
    let result: Response<types::Tokens> = external_client.refresh(request).await.unwrap();
    let tokens = result.into_inner();
    let parser = JWTTokenParser::new(test_helper::PUBLIC_KEY.to_owned());
    assert!(matches!(parser.get_player_id(tokens.session), Result::Ok(value) if value==player));
}
