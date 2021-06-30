use std::str::FromStr;

use ipnetwork::IpNetwork;
use tonic::metadata::MetadataMap;
use tonic::{Request, Response};

use crate::proto::auth::cerberus::internal::cerberus_client::CerberusClient;
use crate::proto::auth::cerberus::internal::*;
use crate::proto::auth::cerberus::types::*;

pub mod cookie;
pub mod google;

async fn create_cerberus_token(
    cerberus_internal_url: String,
    player: u64,
    device_id: String,
) -> Result<Response<Tokens>, tonic::Status> {
    CerberusClient::connect(cerberus_internal_url)
        .await
        .unwrap()
        .create(Request::new(CreateTokenRequest { player, device_id }))
        .await
}

fn get_client_ip(metadata: &MetadataMap) -> IpNetwork {
    let peer_ip = match metadata.get("X-Forwarded-For") {
        None => None,
        Some(x_forwarder_for) => match x_forwarder_for.to_str() {
            Ok(value) => match ipnetwork::IpNetwork::from_str(value) {
                Ok(value) => Some(value),
                Err(_) => None,
            },
            Err(_) => None,
        },
    };

    peer_ip.unwrap_or_else(|| IpNetwork::from_str("127.0.0.1").unwrap())
}
