use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use fnv::FnvBuildHasher;

use cheetah_relay_common::commands::command::{C2SCommandUnion, C2SCommandWithMeta, S2CCommandUnion};
use cheetah_relay_common::commands::command::event::EventCommand;
use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::structure::StructureCommand;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandDescription};
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::client::Client;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, Server2ClientFFIConverter};

pub type ClientId = u16;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
///
#[derive(Debug)]
pub struct Clients {
	clients: HashMap<ClientId, ClientAPI, FnvBuildHasher>,
	client_generator_id: ClientId,
	s2c_command_ffi: Command,
}

#[derive(Debug)]
pub enum ClientsErrors {
	ClientNotFound(u16),
}

#[derive(Debug)]
pub struct ClientAPI {
	pub out_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	pub in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	sender: Sender<ClientRequest>,
}

#[derive(Debug)]
pub enum ClientRequest {
	SetProtocolTimeOffset(Duration),
	Close,
}


impl Drop for ClientAPI {
	fn drop(&mut self) {
		match self.sender.send(ClientRequest::Close) {
			Ok(_) => {
				self.handler.take().unwrap().join().unwrap();
			}
			Err(_) => {}
		}
	}
}

impl Default for Clients {
	fn default() -> Self {
		Clients {
			clients: Default::default(),
			client_generator_id: Default::default(),
			s2c_command_ffi: Default::default(),
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
				
				let client_api = ClientAPI {
					out_commands: out_commands_cloned,
					in_commands: in_commands_cloned,
					handler: Option::Some(handler),
					state: state_cloned,
					sender,
				};
				self.client_generator_id += 1;
				let current_generator_id = self.client_generator_id;
				self.clients.insert(current_generator_id, client_api);
				
				log::info!("Clients::create connection with id {}", current_generator_id);
				Result::Ok(current_generator_id)
			}
			Err(_) => {
				Result::Err(())
			}
		}
	}
	
	pub fn destroy_client(
		&mut self,
		client_id: u16,
	) -> bool {
		match self.clients.remove(&client_id) {
			None => {
				log::error!("Clients::destroy connection with id {} not found", client_id);
				true
			}
			Some(_) => {
				log::trace!("Clients::destroy connection {}", client_id);
				false
			}
		}
	}
	
	pub fn send_command_to_server(
		&mut self,
		client_id: u16,
		command: &Command) -> Result<(), ClientsErrors> {
		match self.clients.get_mut(&client_id) {
			None => {
				Result::Err(ClientsErrors::ClientNotFound(client_id))
			}
			Some(client_api) => {
				let client_command = match command.command_type_c2s {
					C2SCommandFFIType::Create => CreateGameObjectCommand::from_ffi(command),
					C2SCommandFFIType::IncrementLongCounter => IncrementLongC2SCommand::from_ffi(command),
					C2SCommandFFIType::IncrementFloatCounter => IncrementFloat64C2SCommand::from_ffi(command),
					C2SCommandFFIType::Structure => StructureCommand::from_ffi(command),
					C2SCommandFFIType::Event => EventCommand::from_ffi(command),
					C2SCommandFFIType::Unload => DeleteGameObjectCommand::from_ffi(command),
					C2SCommandFFIType::SetLongCounter => SetLongCommand::from_ffi(command),
					C2SCommandFFIType::SetFloatCounter => SetFloat64Command::from_ffi(command),
					C2SCommandFFIType::LoadRoom => { C2SCommandUnion::LoadRoom }
				};
				
				log::info!("schedule command to server {:?}", client_command);
				
				let meta = C2SMetaCommandInformation { timestamp: command.meta_timestamp };
				let command = ApplicationCommandDescription {
					channel: command.to_channel(),
					command: ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
						meta,
						command: client_command,
					}),
				};
				
				client_api.out_commands.lock().unwrap().push_front(command);
				Result::Ok(())
			}
		}
	}
	
	
	pub fn collect_s2c_commands<F>(
		&mut self,
		client_id: ClientId,
		mut collector: F,
	) -> Result<(), ClientsErrors> where F: FnMut(&Command) {
		match self.clients.get(&client_id) {
			None => { Result::Err(ClientsErrors::ClientNotFound(client_id)) }
			Some(client) => {
				let commands = &mut client.in_commands.lock().unwrap();
				let cloned_commands: Vec<_> = commands.drain(..).collect();
				drop(commands); // снимаем lock, так как при вызове функции collector() возможна ситуация deadlock
				let command_ffi = &mut self.s2c_command_ffi;
				cloned_commands.into_iter().for_each(|command| {
					if let ApplicationCommand::S2CCommandWithMeta(command) = command.command {
						log::info!("receive command from server {:?}", command);
						match command.command {
							S2CCommandUnion::Create(command) => { command.to_ffi(command_ffi) }
							S2CCommandUnion::SetLong(command) => { command.to_ffi(command_ffi) }
							S2CCommandUnion::SetFloat64(command) => { command.to_ffi(command_ffi) }
							S2CCommandUnion::SetStruct(command) => { command.to_ffi(command_ffi) }
							S2CCommandUnion::Event(command) => { command.to_ffi(command_ffi) }
							S2CCommandUnion::Delete(command) => { command.to_ffi(command_ffi) }
						};
						command_ffi.meta_timestamp = command.meta.timestamp;
						command_ffi.meta_source_client = command.meta.user_public_key;
						collector(command_ffi);
					}
				});
				Result::Ok(())
			}
		}
	}
	
	pub fn get_connection_status(&self, client_id: ClientId) -> Result<ConnectionStatus, ClientsErrors> {
		match self.clients.get(&client_id) {
			Some(client) => {
				Result::Ok(*client.state.lock().unwrap())
			}
			None => { Result::Err(ClientsErrors::ClientNotFound(client_id)) }
		}
	}
	
	
	pub fn set_protocol_time_offset(&self, client_id: ClientId, time_offset: Duration) {
		if let Some(client) = self.clients.get(&client_id) {
			client.sender.send(ClientRequest::SetProtocolTimeOffset(time_offset)).unwrap();
		}
	}
}
