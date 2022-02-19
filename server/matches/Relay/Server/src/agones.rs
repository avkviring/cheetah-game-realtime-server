use rymder::GameServer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use thiserror::Error;

use crate::registry::client::RegistryClient;
use crate::registry::proto::registry::RelayState;
use crate::registry::proto::registry::{Addr, RelayAddrs};
use cheetah_microservice::tonic::codegen::Arc;

use crate::server::manager::ServerManager;

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
/// Взаимодействие с AGONES SDK
/// Если Agones  не запущен - то relay будет остановлен
///
pub async fn run_agones_cycle(halt_signal: Arc<AtomicBool>, relay_server: Arc<Mutex<ServerManager>>) {
	if std::env::var("ENABLE_AGONES").is_err() {
		return;
	}
	tracing::info!("Agones: Starting");
	match rymder::Sdk::connect(
		None,
		Option::Some(Duration::from_secs(2)),
		Option::Some(Duration::from_secs(2)),
	)
	.await
	{
		Ok((mut sdk, gameserver)) => {
			tracing::info!("Agones: Connected to SDK");
			// сервер готов к работе
			sdk.mark_ready().await.unwrap();
			tracing::info!("Agones: invoked sdk.mark_ready");

			let mut health = sdk.health_check();

			let mut allocated = false;

			while !halt_signal.load(Ordering::Relaxed) {
				// при создании первой комнаты - вызываем allocate
				if !allocated && relay_server.lock().unwrap().created_room_counter > 0 {
					sdk.allocate().await.unwrap();
					tracing::info!("Agones: invoked allocated");
					allocated = true;
				}

				if allocated {
					// todo(v.zakharov): handle error
					notify_registry(&gameserver, RelayState::Allocated).await.unwrap();
				} else {
					// todo(v.zakharov): handle error
					notify_registry(&gameserver, RelayState::Ready).await.unwrap();
				}

				// подтверждаем что сервер жив
				match health.send(()).await {
					Ok(_) => {
						tracing::info!("Agones: invoked health");
					}
					Err(e) => {
						tracing::error!("Agones: health receiver was closed {:?}", e);
						health = sdk.health_check();
					}
				}

				tokio::time::sleep(Duration::from_secs(2)).await;
			}
			// todo(v.zakharov): handle error
			notify_registry(&gameserver, RelayState::NotReady).await.unwrap();
			sdk.shutdown().await.unwrap();
		}
		Err(e) => {
			tracing::error!("Agones: Fail connect {:?}", e);
			panic!("Agones: Fail connect {:?}", e);
		}
	}
}

async fn notify_registry(gs: &GameServer, state: RelayState) -> Result<(), RegistryError> {
	// todo(v.zakharov): do not reconnect every time
	let registry_url = cheetah_microservice::get_internal_srv_uri_from_env("CHEETAH_MATCHES_REGISTRY");
	let client = RegistryClient::new(registry_url).await.map_err(RegistryError::from)?;

	let status = gs.status.as_ref().ok_or(RegistryError::InvalidGameServerStatus(
		"could not find status in GameServer".to_string(),
	))?;
	let host = status.address;
	let port =
		status
			.ports
			.iter()
			.find(|p| p.name == "default")
			.map(|p| p.port)
			.ok_or(RegistryError::InvalidGameServerStatus(
				"could not find port default in GameServer Status".to_string(),
			))?;

	let addrs = RelayAddrs {
		game: Some(Addr {
			host: host.to_string(),
			port: port.into(),
		}),
		grpc_internal: Some(Addr {
			host: cheetah_microservice::get_env("POD_IP"),
			port: cheetah_microservice::get_internal_service_port() as u32,
		}),
	};

	client.update_relay_status(addrs, state).await.map_err(RegistryError::from)
}
