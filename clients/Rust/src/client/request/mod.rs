use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use crate::client::{Client, NetworkStatus};
use crate::client::command::{C2SCommandUnion, S2CCommandUnion};
use crate::log::Logger;

pub enum ClientRequestType {
	GetS2CCommands(Sender<Vec<S2CCommandUnion>>),
	SendCommandToServer(C2SCommandUnion),
	GetConnectionStatus(Sender<NetworkStatus>),
	Close,
}


///
/// Обработка внешних запросов
///
#[derive(Debug)]
pub struct ExternalRequestProcessor {
	receiver: Receiver<ClientRequestType>
}

impl ExternalRequestProcessor {
	pub fn new(receiver: Receiver<ClientRequestType>) -> Self {
		ExternalRequestProcessor {
			receiver
		}
	}
	
	pub fn cycle(&mut self, client: &mut Client) {
		let result = self.receiver.recv_timeout(Duration::from_nanos(100));
		match result {
			Ok(request) => {
				match request {
					ClientRequestType::GetS2CCommands(response) => {
						let commands = client.get_commands_from_server();
						response.send(commands);
					}
					ClientRequestType::SendCommandToServer(command) => {
						client.schedule_command_to_server(command);
					}
					ClientRequestType::GetConnectionStatus(response) => {
						response.send(client.network_status.clone());
					}
					ClientRequestType::Close => {
						client.close();
					}
				}
			}
			Err(e) => {
				// все нормально, просто нет сообщений
			}
		}
	}
}
