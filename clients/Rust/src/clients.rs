use std::collections::HashMap;
use std::sync::mpsc::{Sender, SendError};
use std::thread;
use std::thread::JoinHandle;

use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::SetStructCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectC2SCommand;
use cheetah_relay_common::network::hash::HashValue;

use crate::client::command::S2CCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, Server2ClientFFIConverter};
use crate::client::NetworkStatus;
use crate::client::request::ClientRequestType;
use crate::client::thread::ClientThread;
use crate::log::Logger;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
pub struct Clients {
	clients: HashMap<u16, ClientAPI>,
	client_generator_id: u16,
	s2c_command_ffi: CommandFFI,
}

pub enum ClientsErrors {
	ClientNotFound,
	CollectS2CCommand,
	GetConnectionStatus,
	SendCommandToServerError,
}

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
		if let Some(client) = self.clients.remove(&client_id) {
			match client.sender.send(ClientRequestType::Close) {
				Ok(_) => { true }
				Err(e) => {
					log::error!("destroy_client error in channel {:?}", e);
					false
				}
			}
		} else {
			log::error!("destroy_client client not found");
			false
		}
	}
	
	pub fn send_command_to_server(
		&mut self,
		client_id: u16,
		command: &CommandFFI,
	) -> Result<(), ClientsErrors> {
		match self.clients.get(&client_id) {
			None => {
				Result::Err(ClientsErrors::ClientNotFound)
			}
			Some(client) => {
				let command = match command.command_type_c2s {
					C2SCommandFFIType::Upload => { UploadGameObjectC2SCommand::from_ffi(command) }
					C2SCommandFFIType::IncrementLongCounter => { IncrementLongCounterC2SCommand::from_ffi(command) }
					C2SCommandFFIType::IncrementFloatCounter => { IncrementFloatCounterC2SCommand::from_ffi(command) }
					C2SCommandFFIType::SetStruct => { SetStructCommand::from_ffi(command) }
					C2SCommandFFIType::SendEvent => { EventCommand::from_ffi(command) }
					C2SCommandFFIType::Unload => { UnloadGameObjectCommand::from_ffi(command) }
					C2SCommandFFIType::SetLongCounter => { SetLongCounterCommand::from_ffi(command) }
					C2SCommandFFIType::SetFloatCounter => { SetFloatCounterCommand::from_ffi(command) }
				};
				match client.sender.send(ClientRequestType::SendCommandToServer(command)) {
					Ok(_) => {
						Result::Ok(())
					}
					Err(_) => {
						Result::Err(ClientsErrors::SendCommandToServerError)
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
			None => { Result::Err(ClientsErrors::ClientNotFound) }
			Some(client) => {
				let (sender, receiver) = std::sync::mpsc::channel();
				match client.sender.send(ClientRequestType::GetS2CCommands(sender)) {
					Ok(_) => {
						match receiver.recv() {
							Ok(commands) => {
								let command_ffi = &mut self.s2c_command_ffi;
								commands.into_iter().for_each(|command| {
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
								log::error!("collect commands from server error in receive {:?}", e);
								Result::Err(ClientsErrors::CollectS2CCommand)
							}
						}
					}
					Err(e) => {
						log::error!("collect commands from server error in send {:?}", e);
						Result::Err(ClientsErrors::CollectS2CCommand)
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
						Result::Err(ClientsErrors::GetConnectionStatus)
					}
				}
			}
			None => { Result::Err(ClientsErrors::ClientNotFound) }
		}
	}
}
