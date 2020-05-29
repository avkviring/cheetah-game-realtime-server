use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

use crate::client::Client;
use crate::client::command::{C2SCommandUnion, S2CCommandUnion};
use crate::client::command::event::SendEventC2S;
use crate::client::command::float_counter::IncrementFloatCounterC2S;
use crate::client::command::long_counter::IncrementLongCounterC2S;
use crate::client::command::structure::SetStructC2S;
use crate::client::command::unload::UnloadObjectC2S;
use crate::client::command::upload::UploadObjectC2S;
use crate::client::ffi::{C2SCommandFFIType, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};
use crate::client::ffi::C2SCommandFFIType::IncrementFloatCounter;
use crate::client::request::ClientRequestType;
use crate::log::{ErrorCode, Logger};

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
pub struct Clients {
	clients: HashMap<u16, ClientAPI>,
	client_generator_id: u16,
	s2c_command_ffi: S2CCommandFFI,
}

struct ClientAPI {
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
	pub fn create_client(&mut self, server_address: String) -> u16 {
		let (sender, receiver) = std::sync::mpsc::channel();
		let handler = thread::spawn(move || {
			let mut client = Client::new(server_address, receiver);
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
	) {
		if let Some(client) = self.clients.remove(&client_id) {
			match client.sender.send(ClientRequestType::Close) {
				Ok(_) => {}
				Err(e) => {
					Logger::log_error(ErrorCode::DestroyClient, format!("{:?}", e))
				}
			}
		};
	}
	
	pub fn send_command_to_server(
		&mut self,
		client_id: u16,
		command: S2CCommandFFI,
	) {
		if let Some(client) = self.clients.get(&client_id) {
			let command = match command.c2s_command_type {
				C2SCommandFFIType::Upload => { UploadObjectC2S::from(command) }
				C2SCommandFFIType::IncrementLongCounter => { IncrementLongCounterC2S::from(command) }
				C2SCommandFFIType::IncrementFloatCounter => { IncrementFloatCounterC2S::from(command) }
				C2SCommandFFIType::SetStruct => { SetStructC2S::from(command) }
				C2SCommandFFIType::SendEvent => { SendEventC2S::from(command) }
				C2SCommandFFIType::Unload => { UnloadObjectC2S::from(command) }
			};
			client.sender.send(ClientRequestType::SendCommandToServer(command));
		}
	}
	
	
	pub fn collect_s2c_commands(
		&mut self,
		client_id: u16,
		collector: fn(&S2CCommandFFI),
	) {
		if let Some(client) = self.clients.get(&client_id) {
			let (sender, receiver) = std::sync::mpsc::channel();
			
			match client.sender.send(ClientRequestType::GetS2CCommands(sender)) {
				Ok(_) => {
					match receiver.recv() {
						Ok(commands) => {
							let command_ffi = &mut self.s2c_command_ffi;
							commands.into_iter().for_each(|command| {
								match command {
									S2CCommandUnion::Upload(command) => { command.collect(command_ffi) }
									S2CCommandUnion::SetLongCounter(command) => { command.collect(command_ffi) }
									S2CCommandUnion::SetFloatCounter(command) => { command.collect(command_ffi) }
									S2CCommandUnion::SetStruct(command) => { command.collect(command_ffi) }
									S2CCommandUnion::ReceiveEvent(command) => { command.collect(command_ffi) }
									S2CCommandUnion::Unload(command) => { command.collect(command_ffi) }
								};
								collector(command_ffi);
							})
						}
						Err(e) => {
							Logger::log_error(ErrorCode::CollectS2cCommand, format!("{:?}", e))
						}
					}
				}
				Err(e) => {
					Logger::log_error(ErrorCode::CollectS2cCommand, format!("{:?}", e))
				}
			}
		};
	}
}
