use sqlx::PgPool;

use crate::users::IpNetwork;

pub mod cookie;
pub mod google;
pub mod grpc;
pub mod postgresql;
pub mod proto;
pub mod tokens;
pub mod users;

pub fn get_client_ip(metadata: &tonic::metadata::MetadataMap) -> IpNetwork {
	metadata
		.get("X-Forwarded-For")
		.and_then(|x_forwarder_for| x_forwarder_for.to_str().ok())
		.and_then(|peer_ip| peer_ip.parse().ok())
		.unwrap_or_else(|| "127.0.0.1".parse().unwrap())
}
