use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use rymder::GameServer;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::agones::client::RegistryClient;
use crate::agones::proto::status::{Addr, State};
use crate::env::{get_env, get_internal_grpc_service_default_port, get_internal_srv_uri_from_env};
use crate::server::manager::RoomsServerManager;

pub mod client;
pub mod proto;

#[derive(Error, Debug)]
pub enum RegistryError {
	#[error(transparent)]
	RegistryUnavailable(#[from] tonic::transport::Error),
	#[error(transparent)]
	UpdateRelayStatusFailed(#[from] tonic::Status),
	#[error("Agones GameServer status is invalid: {0}")]
	InvalidGameServerStatus(String),
}

///
/// Цикл оповещения agones и NotifyService
///
pub async fn run_agones(server_manager: Arc<Mutex<RoomsServerManager>>, max_rooms: usize) {
	tracing::debug!("Agones agones sdk");
	match rymder::Sdk::connect(None, Some(Duration::from_secs(2)), Some(Duration::from_secs(2))).await {
		Ok((mut sdk, gameserver)) => {
			tracing::debug!("Agones: Connected to SDK");
			// сервер готов к работе
			sdk.mark_ready().await.unwrap();
			tracing::debug!("Agones: invoked sdk.mark_ready");

			let mut health = sdk.health_check();

			let mut allocated = false;

			while is_server_running(&server_manager).await {
				// при создании первой комнаты - вызываем allocate
				if !allocated && server_manager.lock().await.created_room_counter > 0 {
					sdk.allocate().await.unwrap();
					tracing::debug!("Agones: invoked allocated");
					allocated = true;
				}

				let state = if allocated {
					if server_manager.lock().await.created_room_counter >= max_rooms {
						State::NotReady
					} else {
						State::Allocated
					}
				} else {
					State::Ready
				};
				notify_registry_with_tracing_error(&gameserver, state).await;

				// подтверждаем что сервер жив
				match health.send(()).await {
					Ok(_) => {
						tracing::debug!("Agones: invoked health");
					}
					Err(e) => {
						tracing::error!("Agones: health receiver was closed {:?}", e);
						health = sdk.health_check();
					}
				}

				tokio::time::sleep(Duration::from_secs(2)).await;
			}
			notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
			sdk.shutdown().await.unwrap();
		}
		Err(e) => {
			tracing::error!("Agones: Fail connect {:?}", e);
			panic!("Agones: Fail connect {e:?}");
		}
	}
}

async fn is_server_running(server_manager: &Arc<Mutex<RoomsServerManager>>) -> bool {
	!server_manager.lock().await.get_halt_signal().load(Ordering::Relaxed)
}

async fn notify_registry_with_tracing_error(gs: &GameServer, state: State) -> () {
	match notify_registry(gs, state).await {
		Ok(_) => {}
		Err(e) => {
			tracing::error!("Error notify registry {:?}", e);
		}
	}
}

async fn notify_registry(gs: &GameServer, state: State) -> Result<(), RegistryError> {
	let registry_url = get_internal_srv_uri_from_env("CHEETAH_SERVER_STATUS_RECEIVER");
	let client = RegistryClient::new(registry_url).await.map_err(RegistryError::from)?;

	let status = gs
		.status
		.as_ref()
		.ok_or_else(|| RegistryError::InvalidGameServerStatus("could not find status in GameServer".to_owned()))?;
	let host = status.address;
	let port = status
		.ports
		.iter()
		.find(|p| p.name == "default")
		.map(|p| p.port)
		.ok_or_else(|| RegistryError::InvalidGameServerStatus("could not find port default in GameServer Status".to_owned()))?;

	let game = Addr {
		host: host.to_string(),
		port: port.into(),
	};
	let grpc_internal = Addr {
		host: get_env("POD_IP"),
		port: u32::from(get_internal_grpc_service_default_port()),
	};

	client.update_server_status(game, grpc_internal, state).await.map_err(RegistryError::from)
}
