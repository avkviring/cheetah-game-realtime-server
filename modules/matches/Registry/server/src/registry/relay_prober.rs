use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;
use tonic::transport::Endpoint;
use tonic::Request;

use crate::proto::matches::realtime::internal::realtime_client::RealtimeClient;
use crate::proto::matches::realtime::internal::ProbeRequest;

#[async_trait]
pub trait RelayProber: Send + Sync {
	async fn probe(&self, addr: SocketAddr) -> Result<(), ProbeError>;
}

#[derive(Error, Debug)]
pub enum ProbeError {
	#[error(transparent)]
	TransportError(#[from] tonic::transport::Error),

	#[error(transparent)]
	GrpcErrorStatus(#[from] tonic::Status),
}

pub struct ReconnectProber {}

#[async_trait]
impl RelayProber for ReconnectProber {
	// todo(v.zakharov): add health-check rpc instead of opening connection each time
	async fn probe(&self, addr: SocketAddr) -> Result<(), ProbeError> {
		let mut builder = Endpoint::from_str(&format!("http://{}", addr)).unwrap();
		builder = builder.connect_timeout(Duration::from_secs(1));
		let mut client = RealtimeClient::connect(builder)
			.await
			.map_err(ProbeError::from)?;

		client
			.probe(Request::new(ProbeRequest {}))
			.await
			.map_err(ProbeError::from)?;

		Ok(())
	}
}
