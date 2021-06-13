use tonic::transport::*;
use tonic::Request;

use cerberus_client::CerberusClient;
use proto::authentication::external::cookie;
use proto::cerberus::external::{cerberus_client, RefreshTokenRequest};
use proto::cerberus::types::Tokens;

pub mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let url = "https://test.dev.cheetah.games";
    let url = "http://localhost:7777";
    let channel = Channel::from_static(url).connect().await?;
    let tokens = test_authentication_service(channel.clone()).await;
    test_cerberus_service(channel.clone(), tokens).await;
    Result::Ok(())
}

async fn test_cerberus_service(channel: Channel, tokens: Tokens) {
    let mut client = CerberusClient::new(channel);
    let request = Request::new(RefreshTokenRequest {
        token: tokens.refresh,
    });
    let response = client.refresh(request).await;
    response.expect("cerberus service error");
    println!("Test cerberus ... OK");
}

async fn test_authentication_service(channel: Channel) -> Tokens {
    let mut client = cookie::cookie_client::CookieClient::new(channel);
    let request = Request::new(cookie::RegistryRequest {
        device_id: "device-id".to_owned(),
    });
    let response = client.register(request).await;
    let response = response.expect("authentication service error");
    println!("Test authentication ... OK");
    return response.get_ref().tokens.clone().unwrap();
}
