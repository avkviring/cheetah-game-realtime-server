use tonic::transport::*;
use tonic::{Request, Response, Status};

use proto::authentication::external::cookie;

pub mod proto;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://test.dev.cheetah.games";
    let channel = Channel::from_static(url).connect().await?;

    let mut client = cookie::cookie_client::CookieClient::new(channel);

    let response = client
        .register(Request::new(cookie::RegistryRequest {
            device_id: "device-id".to_owned(),
        }))
        .await;

    match response {
        Ok(response) => {
            println!("response cookie {}", response.get_ref().cookie);
        }
        Err(err) => {
            println!("response error {}", err.message());
        }
    }

    Result::Ok(())
}
