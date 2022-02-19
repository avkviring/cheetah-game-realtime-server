use crate::proto::matches::registry::internal::registry_server::Registry;
use crate::proto::matches::registry::internal::{
	FindFreeRelayRequest, FindFreeRelayResponse, RelayState, RelayStatusUpdate, UpdateRelayStatusResponse,
};
use crate::registry::relay_finder::RelayFinder;
use crate::registry::relay_prober::ReconnectProber;
use crate::registry::storage::{RedisStorage, Storage, StorageError};
use std::convert::TryInto;
use thiserror::Error;
use tonic::{Request, Response, Status};

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
		Result::Ok(registry_service)
	}
}

#[tonic::async_trait]
impl Registry for RegistryService {
	async fn find_free_relay(&self, _request: Request<FindFreeRelayRequest>) -> Result<Response<FindFreeRelayResponse>, Status> {
		let addrs = self.free_relay_provider.get_random_relay_addr().await.map_err(|err| {
			match err {
				StorageError::NoRelayFound => tracing::warn!("could not find free relay"),
				StorageError::MalformedValue(ref e) => tracing::error!("storage value corrupted: {:?}", e),
				StorageError::RedisError(ref e) => tracing::warn!("redis error: {:?}", e),
			};
			Status::unavailable(format!("Error: {:?}", err))
		})?;

		Ok(Response::new(FindFreeRelayResponse {
			addrs: Some(addrs.into()),
		}))
	}

	async fn update_relay_status(
		&self,
		request: tonic::Request<RelayStatusUpdate>,
	) -> Result<tonic::Response<UpdateRelayStatusResponse>, Status> {
		let msg = request.into_inner();

		let addrs = msg.addrs.try_into().map_err(|e| {
			let msg = format!("received malformed RelayAddrs: {:?}", e);
			tracing::error!("{}", msg);
			Status::invalid_argument(msg)
		})?;

		let msg_state = msg.state;
		let state = RelayState::from_i32(msg_state).ok_or_else(|| {
			let msg = format!("received unknown RelayState: {:?}", msg_state);
			tracing::error!("{}", msg);
			Status::invalid_argument(msg)
		})?;

		match self.storage.update_status(&addrs, state).await {
			Ok(_) => Ok(tonic::Response::new(UpdateRelayStatusResponse::default())),
			Err(err) => {
				let msg = format!("could not save status to storage: {:?}", err);
				tracing::warn!("{}", msg);
				Err(Status::unavailable(msg))
			}
		}
	}
}
