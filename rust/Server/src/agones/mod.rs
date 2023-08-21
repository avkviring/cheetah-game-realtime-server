use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use rymder::{GameServer, Sdk};
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::agones::client::RegistryClient;
use crate::agones::proto::status::{Addr, State};
use crate::env::{get_env, get_internal_grpc_service_default_port, get_internal_srv_uri_from_env};
use crate::server::manager::ServerManager;

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
pub async fn agones_und_notifyservice_cycle(server_manager: Arc<Mutex<ServerManager>>, max_alive_rooms: usize, max_created_rooms: usize) {
	match Sdk::connect(None, Some(Duration::from_secs(2)), Some(Duration::from_secs(2))).await {
		Ok((mut sdk, gameserver)) => {
			sdk.mark_ready().await.unwrap();

			let mut health = sdk.health_check();
			let mut allocated = false;

			while is_server_running(&server_manager).await {
				let server_manager = server_manager.lock().await;
				let current_count_rooms = server_manager.get_rooms().unwrap_or_default().len();
				let created_rooms_count = server_manager.get_created_rooms_count().unwrap();

				if !allocated && current_count_rooms > 0 {
					sdk.allocate().await.unwrap();
					allocated = true;
				}

				let need_to_restart = created_rooms_count >= max_created_rooms;
				let is_allow_to_restart = need_to_restart && current_count_rooms == 0;
				if is_allow_to_restart {
					tracing::info!("Max created rooms limit reached - shutdown");
					sdk.shutdown().await.unwrap();
					notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
				} else {
					let server_is_full = current_count_rooms >= max_alive_rooms;
					let state = if allocated {
						if server_is_full || need_to_restart {
							State::NotReady
						} else {
							State::Allocated
						}
					} else {
						State::Ready
					};
					notify_registry_with_tracing_error(&gameserver, state).await;
					health = send_agones_health(&mut sdk, health).await;
				}
				tokio::time::sleep(Duration::from_secs(2)).await;
			}
			notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
			sdk.shutdown().await.unwrap();
		}
		Err(e) => {
			panic!("Agones: Fail connect {e:?}");
		}
	}
}

async fn send_agones_health(sdk: &mut Sdk, health: Sender<()>) -> Sender<()> {
	let result = match health.send(()).await {
		Ok(_) => health,
		Err(e) => {
			tracing::error!("Agones: health receiver was closed {:?}", e);
			sdk.health_check()
		}
	};
	tracing::debug!("Agones: sended agones health signal");
	result
}

async fn is_server_running(server_manager: &Arc<Mutex<ServerManager>>) -> bool {
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
