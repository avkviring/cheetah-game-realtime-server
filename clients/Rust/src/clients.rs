use std::collections::HashMap;
use std::sync::mpsc::{RecvError, Sender, SendError};
use std::thread;
use std::thread::JoinHandle;

use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectC2SCommand;
use cheetah_relay_common::network::hash::HashValue;

use crate::client::command::S2CCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, Server2ClientFFIConverter};
use crate::client::NetworkStatus;
use crate::client::request::ClientRequestType;
use crate::client::thread::ClientThread;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
///
#[derive(Debug)]
pub struct Clients {
	clients: HashMap<u16, ClientAPI>,
	client_generator_id: u16,
	s2c_command_ffi: CommandFFI,
}

#[derive(Debug)]
pub enum ClientsErrors {
	ClientNotFound(u16),
	CollectS2CCommand(String),
	GetConnectionStatus(RecvError),
	SendCommandToServerError(SendError<ClientRequestType>),
}

#[derive(Debug)]
pub struct ClientAPI {
	sender: Sender<ClientRequestType>,
	handler: JoinHandle<()>,
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
	pub fn create_client(&mut self,
						 server_address: String,
						 room_hash: HashValue,
						 client_hash: HashValue,
	) -> u16 {
		let (sender, receiver) = std::sync::mpsc::channel();
		let handler = thread::spawn(move || {
			let mut client = ClientThread::new(server_address, room_hash, client_hash, receiver);
			client.run()
		});
		let client_api = ClientAPI {
			sender,
			handler,
		};
		
		self.client_generator_id += 1;
		let current_generator_id = self.client_generator_id;
		self.clients.insert(current_generator_id.clone(), client_api);
		current_generator_id
	}
	
	pub fn destroy_client(
		&mut self,
		client_id: u16,
	) -> bool {
		match self.clients.remove(&client_id) {
			None => {
				log::error!("Clients::destroy_client client with id {} not found", client_id);
				true
			}
			Some(_) => {
				log::trace!("Clients::destroy_client client {}", client_id);
				false
			}
		}
	}
	
	pub fn send_command_to_server(
		&mut self,
		client_id: u16,
		command: &CommandFFI,
	) -> Result<(), ClientsErrors> {
		match self.clients.get(&client_id) {
			None => {
				Result::Err(ClientsErrors::ClientNotFound(client_id))
			}
			Some(client) => {
				let command = match command.command_type_c2s {
					C2SCommandFFIType::Upload => { UploadGameObjectC2SCommand::from_ffi(command) }
					C2SCommandFFIType::IncrementLongCounter => { IncrementLongCounterC2SCommand::from_ffi(command) }
					C2SCommandFFIType::IncrementFloatCounter => { IncrementFloatCounterC2SCommand::from_ffi(command) }
					C2SCommandFFIType::Structure => { StructureCommand::from_ffi(command) }
					C2SCommandFFIType::Event => { EventCommand::from_ffi(command) }
					C2SCommandFFIType::Unload => { UnloadGameObjectCommand::from_ffi(command) }
					C2SCommandFFIType::SetLongCounter => { SetLongCounterCommand::from_ffi(command) }
					C2SCommandFFIType::SetFloatCounter => { SetFloatCounterCommand::from_ffi(command) }
				};
				
				if log::log_enabled!(log::Level::Info) {
					log::info!("schedule command to server {:?}", command);
				}
				
				match client.sender.send(ClientRequestType::SendCommandToServer(command)) {
					Ok(_) => {
						Result::Ok(())
					}
					Err(e) => {
						Result::Err(ClientsErrors::SendCommandToServerError(e))
					}
				}
			}
		}
	}
	
	
	pub fn collect_s2c_commands<F>(
		&mut self,
		client_id: u16,
		mut collector: F,
	) -> Result<(), ClientsErrors> where F: FnMut(&CommandFFI) -> () {
		match self.clients.get(&client_id) {
			None => { Result::Err(ClientsErrors::ClientNotFound(client_id)) }
			Some(client) => {
				let (sender, receiver) = std::sync::mpsc::channel();
				match client.sender.send(ClientRequestType::GetS2CCommands(sender)) {
					Ok(_) => {
						match receiver.recv() {
							Ok(commands) => {
								let command_ffi = &mut self.s2c_command_ffi;
								commands.into_iter().for_each(|command| {
									if log::log_enabled!(log::Level::Info) {
										log::info!("receive command from server {:?}", command);
									}
									match command {
										S2CCommandUnion::Upload(command) => { command.to_ffi(command_ffi) }
										S2CCommandUnion::SetLongCounter(command) => { command.to_ffi(command_ffi) }
										S2CCommandUnion::SetFloatCounter(command) => { command.to_ffi(command_ffi) }
										S2CCommandUnion::SetStruct(command) => { command.to_ffi(command_ffi) }
										S2CCommandUnion::Event(command) => { command.to_ffi(command_ffi) }
										S2CCommandUnion::Unload(command) => { command.to_ffi(command_ffi) }
									};
									collector(command_ffi);
								});
								Result::Ok(())
							}
							Err(e) => {
								Result::Err(ClientsErrors::CollectS2CCommand(format!("{:?}", e)))
							}
						}
					}
					Err(e) => {
						Result::Err(ClientsErrors::CollectS2CCommand(format!("{:?}", e)))
					}
				}
			}
		}
	}
	
	pub fn get_connection_status(&self, client_id: u16) -> Result<NetworkStatus, ClientsErrors> {
		match self.clients.get(&client_id) {
			Some(client) => {
				let (sender, receiver) = std::sync::mpsc::channel();
				client.sender.send(ClientRequestType::GetConnectionStatus(sender)).unwrap();
				match receiver.recv() {
					Ok(status) => {
						Result::Ok(status)
					}
					Err(e) => {
						Result::Err(ClientsErrors::GetConnectionStatus(e))
					}
				}
			}
			None => { Result::Err(ClientsErrors::ClientNotFound(client_id)) }
		}
	}
}
