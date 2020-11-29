use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fnv::FnvBuildHasher;

use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::client::Client;
use crate::controller::ClientController;

pub type ClientId = u16;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
///
pub struct Registry {
	pub controllers: HashMap<ClientId, ClientController, FnvBuildHasher>,
	client_generator_id: ClientId,
	pub current_client: Option<u16>,
}

#[derive(Debug)]
pub enum ClientRequest {
	SetProtocolTimeOffset(Duration),
	Close,
}

impl Default for Registry {
	fn default() -> Self {
		Registry {
			controllers: Default::default(),
			client_generator_id: Default::default(),
			current_client: None,
		}
	}
}

impl Registry {
	pub fn create_client(
		&mut self,
		server_address: String,
		user_public_key: UserPublicKey,
		user_private_key: UserPrivateKey,
		start_frame_id: u64,
	) -> Result<ClientId, ()> {
		let start_frame_id = Arc::new(AtomicU64::new(start_frame_id));
		let out_commands = Arc::new(Mutex::new(VecDeque::new()));
		let in_commands = Arc::new(Mutex::new(VecDeque::new()));
		let state = Arc::new(Mutex::new(ConnectionStatus::Connecting));

		let out_commands_cloned = out_commands.clone();
		let in_commands_cloned = in_commands.clone();
		let state_cloned = state.clone();

		let (sender, receiver) = std::sync::mpsc::channel();
		match Client::new(
			SocketAddr::from_str(server_address.as_str()).unwrap(),
			user_public_key,
			user_private_key,
			out_commands,
			in_commands,
			state,
			receiver,
			start_frame_id.clone(),
		) {
			Ok(client) => {
				let handler = thread::spawn(move || {
					client.run();
				});

				let controller = ClientController::new(
					user_public_key,
					handler,
					state_cloned,
					in_commands_cloned,
					out_commands_cloned,
					sender,
					start_frame_id,
				);
				self.client_generator_id += 1;
				let client_id = self.client_generator_id;
				self.controllers.insert(client_id, controller);

				log::info!("[registry] create client({})", client_id);
				self.current_client = Some(client_id);
				Result::Ok(client_id)
			}
			Err(_) => {
				log::error!("[registry] error create client");
				Result::Err(())
			}
		}
	}

	pub fn destroy_client(&mut self) -> bool {
		match self.current_client {
			None => {
				log::error!("[registry:destroy] current client not set");
				false
			}
			Some(ref current_client) => match self.controllers.remove(current_client) {
				None => {
					log::error!("[registry:destroy] connection with id {} not found", current_client);
					false
				}
				Some(_) => {
					log::trace!("[registry:destroy] connection {}", current_client);
					true
				}
			},
		}
	}
}
