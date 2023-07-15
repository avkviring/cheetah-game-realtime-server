use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use rymder::GameServer;
use thiserror::Error;
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
pub async fn run_agones(server_manager: Arc<Mutex<ServerManager>>, max_rooms: usize) {
	let start_time = SystemTime::now();
	tracing::info!("Agones agones sdk");
	match rymder::Sdk::connect(None, Some(Duration::from_secs(2)), Some(Duration::from_secs(2))).await {
		Ok((mut sdk, gameserver)) => {
			tracing::info!("Agones: Connected to SDK");
			// сервер готов к работе
			sdk.mark_ready().await.unwrap();
			tracing::info!("Agones: invoked sdk.mark_ready");

			let mut health = sdk.health_check();

			let mut allocated = false;

			while is_server_running(&server_manager).await {
				tracing::info!("Agones: cycle start");

				let start_time = SystemTime::now();
				tracing::info!("Agones: start get count rooms");
				let count_rooms = server_manager.lock().await.get_rooms().unwrap_or_default().len();
				tracing::info!("Agones: end get count rooms {:?}", start_time.elapsed().unwrap().as_secs());

				if !allocated && count_rooms > 0 {
					let start_time = SystemTime::now();
					tracing::info!("Agones: start allocate");
					sdk.allocate().await.unwrap();
					tracing::info!("Agones: end allocate {:?}", start_time.elapsed().unwrap().as_secs());
					allocated = true;
				}

				let state = if allocated {
					if count_rooms >= max_rooms {
						State::NotReady
					} else {
						State::Allocated
					}
				} else {
					State::Ready
				};
				tracing::info!("Agones: state {:?}", state);

				let start_time = SystemTime::now();
				tracing::info!("Agones: start notify registry");
				notify_registry_with_tracing_error(&gameserver, state).await;
				tracing::info!("Agones: end notify registry {:?}", start_time.elapsed().unwrap().as_secs());

				let start_time = SystemTime::now();
				tracing::info!("Agones: start notify health");
				// подтверждаем что сервер жив
				match health.send(()).await {
					Ok(_) => {
						tracing::error!("Agones: invoked health {:?}", start_time.elapsed().unwrap().as_secs());
					}
					Err(e) => {
						tracing::error!("Agones: health receiver was closed {:?}", e);
						health = sdk.health_check();
					}
				}
				tracing::info!("Agones: end notify health {:?}", start_time.elapsed().unwrap().as_secs());

				tokio::time::sleep(Duration::from_secs(2)).await;
			}
			tracing::error!("Agones: server stopped");
			notify_registry_with_tracing_error(&gameserver, State::NotReady).await;
			sdk.shutdown().await.unwrap();
			tracing::error!("Agones: schutdown");
		}
		Err(e) => {
			tracing::error!("Agones: Fail connect {:?}", e);
			panic!("Agones: Fail connect {e:?}");
		}
	}
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
