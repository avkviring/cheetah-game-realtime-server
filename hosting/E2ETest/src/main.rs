use std::env;
use std::str::FromStr;

use tonic::Request;
use tonic::transport::*;

use proto::auth::cerberus::types::*;

pub mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let url = args.get(1).unwrap().to_owned();
    println!("run test on cluster {}", &url);
    let channel = Channel::builder(Uri::from_str(url.as_str()).unwrap())
        .connect()
        .await?;
    let tokens = test_authentication_service(channel.clone()).await;
    test_cerberus_service(channel.clone(), tokens).await;
    Result::Ok(())
}

async fn test_cerberus_service(channel: Channel, tokens: Tokens) {
    let mut client = proto::auth::cerberus::external::cerberus_client::CerberusClient::new(channel);
    let request = Request::new(proto::auth::cerberus::external::RefreshTokenRequest {
        token: tokens.refresh,
    });
    let response = client.refresh(request).await;
    response.expect("cerberus service error");
    println!("Test cerberus ... OK");
}

async fn test_authentication_service(channel: Channel) -> Tokens {
    let mut client = proto::auth::cookie::external::cookie_client::CookieClient::new(channel);
    let request = Request::new(proto::auth::cookie::external::RegistryRequest {
        device_id: "device-id".to_owned(),
    });
    let response = client.register(request).await;
    let response = response.expect("authentication service error");
    println!("Test authentication ... OK");
    return response.get_ref().tokens.clone().unwrap();
}
