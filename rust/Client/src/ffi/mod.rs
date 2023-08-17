use std::io;
use std::sync::mpsc::SendError;
use std::sync::Mutex;

use lazy_static::lazy_static;
use thiserror::Error;

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::registry::{ClientId, Registry};
use crate::clients::ClientRequest;

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientError {
	#[error("Create client error {0}")]
	CreateClientError(#[from] io::Error),
	#[error("Registry mutex error {0}")]
	RegistryMutex(String),
	#[error("Client not found {0}")]
	ClientNotFound(ClientId),
	#[error("Connection status mutex error {0}")]
	ConnectionStatusMutexError(String),
	#[error("Send task error {0}")]
	SendTaskError(#[from] SendError<ClientRequest>),
}

impl ClientError {
	pub(crate) fn store_error_and_get_code(&self) -> u8 {
		let mut last_error = LAST_ERROR.lock().unwrap();
		let msg = format!("{self:?}");
		*last_error = msg;

		match self {
			ClientError::RegistryMutex(_) => 1,
			ClientError::ClientNotFound(_) => 2,
			ClientError::ConnectionStatusMutexError { .. } => 3,
			ClientError::SendTaskError { .. } => 4,
			ClientError::CreateClientError(_) => 5,
		}
	}
}

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

pub fn execute<F, R>(body: F) -> u8
where
	F: FnOnce(&mut Registry) -> Result<R, ClientError>,
{
	let mut lock = REGISTRY.lock();
	match lock.as_mut() {
		Ok(registry) => match body(registry) {
			Ok(_) => 0,
			Err(e) => e.store_error_and_get_code(),
		},
		Err(e) => {
			let error = ClientError::RegistryMutex(format!("{e:?}"));
			error.store_error_and_get_code()
		}
	}
}

pub fn execute_with_client<F, R>(client_id: ClientId, action: F) -> u8
where
	F: FnOnce(&mut ApplicationThreadClient) -> Result<R, ClientError>,
{
	execute(|registry| match registry.clients.get_mut(&client_id) {
		None => Err(ClientError::ClientNotFound(client_id)),
		Some(client_api) => action(client_api),
	})
}
