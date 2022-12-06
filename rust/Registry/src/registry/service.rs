use std::convert::TryInto;

use thiserror::Error;
use tonic::{Request, Response, Status};

use cheetah_microservice::trace::Trace;

use crate::proto::matches::registry::internal::registry_server::Registry;
use crate::proto::matches::registry::internal::{
	FindFreeRelayRequest, FindFreeRelayResponse, RelayState, RelayStatusUpdate, UpdateRelayStatusResponse,
};
use crate::registry::relay_finder::RelayFinder;
use crate::registry::relay_prober::ReconnectProber;
use crate::registry::storage::{RedisStorage, Storage, StorageError};

#[derive(Error, Debug)]
pub enum RegistryError {
	#[error(transparent)]
	StorageError(#[from] StorageError),
}

pub struct RegistryService {
	storage: Box<dyn Storage>,
	free_relay_provider: RelayFinder,
}

impl RegistryService {
	pub async fn new(redis_dsn: &str) -> Result<RegistryService, RegistryError> {
		let storage = RedisStorage::new(redis_dsn).await.map_err(RegistryError::from)?;
		let free_relay_provider = RelayFinder::new(Box::new(storage.clone()), Box::new(ReconnectProber {}));
		let registry_service = RegistryService {
			storage: Box::new(storage),
			free_relay_provider,
		};
		Ok(registry_service)
	}
}

#[tonic::async_trait]
impl Registry for RegistryService {
	async fn find_free_relay(&self, _request: Request<FindFreeRelayRequest>) -> Result<Response<FindFreeRelayResponse>, Status> {
		let addrs = self
			.free_relay_provider
			.get_random_relay_addr()
			.await
			.trace_err("Get random relay addr")
			.map_err(Status::internal)?;

		Ok(Response::new(FindFreeRelayResponse { addrs: Some(addrs.into()) }))
	}

	async fn update_relay_status(&self, request: Request<RelayStatusUpdate>) -> Result<Response<UpdateRelayStatusResponse>, Status> {
		let msg = request.into_inner();

		let addrs = msg.addrs.try_into().trace_err("Get relay addr from message").map_err(Status::internal)?;

		let msg_state = msg.state;

		let state = RelayState::from_i32(msg_state)
			.ok_or(())
			.trace_err("Get relayState from i32")
			.map_err(Status::internal)?;

		self.storage
			.update_status(&addrs, state)
			.await
			.trace_err("Update relay status")
			.map_err(Status::internal)?;

		Ok(Response::new(UpdateRelayStatusResponse::default()))
	}
}
