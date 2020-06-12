use std::sync::mpsc::{Receiver, Sender, SendError, TryRecvError};
use std::time::Duration;

use crate::client::{Client, NetworkStatus};
use crate::client::command::{C2SCommandUnion, S2CCommandUnion};

pub enum ClientRequestType {
	GetS2CCommands(Sender<Vec<S2CCommandUnion>>),
	SendCommandToServer(C2SCommandUnion),
	GetConnectionStatus(Sender<NetworkStatus>),
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
	
	pub fn cycle(&mut self, client: &mut Client) -> Result<(), ()> {
		let result = self.receiver.try_recv();
		match result {
			Ok(request) => {
				match request {
					ClientRequestType::GetS2CCommands(response) => {
						let commands = client.get_commands_from_server();
						response.send(commands).map_err(|_| ())
					}
					ClientRequestType::SendCommandToServer(command) => {
						client.schedule_command_to_server(command);
						Result::Ok(())
					}
					ClientRequestType::GetConnectionStatus(response) => {
						response.send(client.network_status.clone()).map_err(|_| ())
					}
				}
			}
			Err(e) => {
				match e {
					TryRecvError::Empty => {
						Result::Ok(())
					}
					TryRecvError::Disconnected => {
						Result::Err(())
					}
				}
			}
		}
	}
}
