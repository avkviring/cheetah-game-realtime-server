use std::convert::TryInto;

use thiserror::Error;
use tonic::{Request, Response, Status};

use cheetah_libraries_microservice::trace::{
	trace_and_convert_to_tonic_internal_status,
	trace_and_convert_to_tonic_unauthenticated_status_with_full_message,
};

use crate::proto::matches::registry::internal::registry_server::Registry;
use crate::proto::matches::registry::internal::{
	FindFreeRelayRequest, FindFreeRelayResponse, RelayState, RelayStatusUpdate,
	UpdateRelayStatusResponse,
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
		let storage = RedisStorage::new(redis_dsn)
			.await
			.map_err(RegistryError::from)?;
		let free_relay_provider =
			RelayFinder::new(Box::new(storage.clone()), Box::new(ReconnectProber {}));
		let registry_service = RegistryService {
			storage: Box::new(storage),
			free_relay_provider,
		};
		Ok(registry_service)
	}
}

#[tonic::async_trait]
impl Registry for RegistryService {
	async fn find_free_relay(
		&self,
		_request: Request<FindFreeRelayRequest>,
	) -> Result<Response<FindFreeRelayResponse>, Status> {
		let addrs = self
			.free_relay_provider
			.get_random_relay_addr()
			.await
			.map_err(trace_and_convert_to_tonic_unauthenticated_status_with_full_message)?;

		Ok(Response::new(FindFreeRelayResponse {
			addrs: Some(addrs.into()),
		}))
	}

	async fn update_relay_status(
		&self,
		request: Request<RelayStatusUpdate>,
	) -> Result<Response<UpdateRelayStatusResponse>, Status> {
		let msg = request.into_inner();

		let addrs = msg
			.addrs
			.try_into()
			.map_err(trace_and_convert_to_tonic_internal_status)?;

		let msg_state = msg.state;
		let state = RelayState::from_i32(msg_state)
			.ok_or_else(|| trace_and_convert_to_tonic_internal_status(""))?;

		self.storage
			.update_status(&addrs, state)
			.await
			.map_err(trace_and_convert_to_tonic_internal_status)?;
		Ok(Response::new(UpdateRelayStatusResponse::default()))
	}
}
