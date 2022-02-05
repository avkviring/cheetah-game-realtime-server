use crate::proto::matches::relay::internal::relay_client::RelayClient;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use thiserror::Error;
use tonic::transport::Endpoint;

#[async_trait]
pub trait RelayProber: Send + Sync {
	async fn probe(&self, addr: SocketAddr) -> Result<(), ProbeError>;
}

#[derive(Error, Debug)]
pub enum ProbeError {
	#[error(transparent)]
	Error(#[from] tonic::transport::Error),
}

pub struct ReconnectProber {}

#[async_trait]
impl RelayProber for ReconnectProber {
	// todo(v.zakharov): add health-check rpc instead of opening connection each time
	async fn probe(&self, addr: SocketAddr) -> Result<(), ProbeError> {
		let mut builder = Endpoint::from_str(&format!("http://{}", addr)).unwrap();
		builder = builder.connect_timeout(Duration::from_secs(1));
		RelayClient::connect(builder).await.map(|_| ()).map_err(ProbeError::from)
	}
}
