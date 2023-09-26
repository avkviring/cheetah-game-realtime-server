use rymder::GameServer;
use thiserror::Error;
use crate::env::{get_env, get_internal_grpc_service_default_port, get_internal_srv_uri_from_env};
use crate::intergration::registry::client::RegistryClient;
use crate::intergration::registry::proto::status::{Addr, State};

pub mod proto;
pub mod client;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error(transparent)]
    RegistryUnavailable(#[from] tonic::transport::Error),
    #[error(transparent)]
    UpdateRelayStatusFailed(#[from] tonic::Status),
    #[error("Agones GameServer status is invalid: {0}")]
    InvalidGameServerStatus(String),
}



pub async fn notify_registry_with_tracing_error(gs: &GameServer, state: State) -> () {
    match notify_registry(gs, state).await {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error notify registry {:?}", e);
        }
    }
}

pub async fn notify_registry(gs: &GameServer, state: State) -> Result<(), RegistryError> {
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

