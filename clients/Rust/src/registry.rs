use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::str::FromStr;
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
pub struct Clients {
	pub controllers: HashMap<ClientId, ClientController, FnvBuildHasher>,
	client_generator_id: ClientId,
	pub current_client: Option<u16>,
}


#[derive(Debug)]
pub enum ClientRequest {
	SetProtocolTimeOffset(Duration),
	Close,
}


impl Default for Clients {
	fn default() -> Self {
		Clients {
			controllers: Default::default(),
			client_generator_id: Default::default(),
			current_client: None,
		}
	}
}


impl Clients {
	pub fn create_client(&mut self, server_address: String, user_public_key: UserPublicKey, user_private_key: UserPrivateKey) -> Result<ClientId, ()> {
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
				);
				self.client_generator_id += 1;
				let client_id = self.client_generator_id;
				self.controllers.insert(client_id, controller);
				
				log::info!("Clients::create connection with id {}", client_id);
				self.current_client = Some(client_id);
				Result::Ok(client_id)
			}
			Err(_) => {
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
			Some(ref current_client) => {
				match self.controllers.remove(current_client) {
					None => {
						log::error!("[registry:destroy] connection with id {} not found", current_client);
						false
					}
					Some(_) => {
						log::trace!("[registry:destroy] connection {}", current_client);
						true
					}
				}
			}
		}
	}
	
	
	// pub fn collect_s2c_commands<F>(
	// 	&mut self,
	// 	client_id: ClientId,
	// 	mut collector: F,
	// ) -> Result<(), ClientsErrors> where F: FnMut(&Command) {
	// 	match self.clients.get(&client_id) {
	// 		None => { Result::Err(ClientsErrors::ClientNotFound(client_id)) }
	// 		Some(client) => {
	// 			let commands = &mut client.in_commands.lock().unwrap();
	// 			let cloned_commands: Vec<_> = commands.drain(..).collect();
	// 			drop(commands); // снимаем lock, так как при вызове функции collector() возможна ситуация deadlock
	// 			let command_ffi = &mut self.s2c_command_ffi;
	// 			cloned_commands.into_iter().for_each(|command| {
	// 				if let ApplicationCommand::S2CCommandWithMeta(command) = command.command {
	// 					log::info!("receive command from server {:?}", command);
	// 					match command.command {
	// 						S2CCommandUnion::Create(command) => { command.to_ffi(command_ffi) }
	// 						S2CCommandUnion::SetLong(command) => { command.to_ffi(command_ffi) }
	// 						S2CCommandUnion::SetFloat64(command) => { command.to_ffi(command_ffi) }
	// 						S2CCommandUnion::SetStruct(command) => { command.to_ffi(command_ffi) }
	// 						S2CCommandUnion::Event(command) => { command.to_ffi(command_ffi) }
	// 						S2CCommandUnion::Delete(command) => { command.to_ffi(command_ffi) }
	// 					};
	// 					command_ffi.meta_timestamp = command.meta.timestamp;
	// 					command_ffi.meta_source_client = command.meta.user_public_key;
	// 					collector(command_ffi);
	// 				}
	// 			});
	// 			Result::Ok(())
	// 		}
	// 	}
	// }
}


