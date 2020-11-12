use std::sync::mpsc::{Receiver, TryRecvError};

use cheetah_relay_common::commands::command::{C2SCommandUnion, C2SCommandWithMeta};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;

use crate::client::Client;

pub enum ClientRequestType {
	SendCommandToServer(C2SCommandUnion, C2SMetaCommandInformation),
	Close,
}


///
/// Обработка внешних запросов
///
#[derive(Debug)]
pub struct ExternalRequestProcessor {
	receiver: Receiver<ClientRequestType>
}

pub enum RequestResult {
	Ok,
	Close,
}

impl ExternalRequestProcessor {
	pub fn new(receiver: Receiver<ClientRequestType>) -> Self {
		ExternalRequestProcessor {
			receiver
		}
	}
	
	pub fn cycle(&mut self, client: &mut Client) -> Result<RequestResult, ()> {
		loop {
			let result = self.receiver.try_recv();
			match result {
				Ok(request) => {
					match request {
						ClientRequestType::SendCommandToServer(command, meta) => {
							client.schedule_command_to_server(C2SCommandWithMeta {command, meta});
						}
						ClientRequestType::Close => {
							return Result::Ok(RequestResult::Close);
						}
					}
				}
				Err(e) => {
					match e {
						TryRecvError::Empty => {
							return Result::Ok(RequestResult::Ok);
						}
						TryRecvError::Disconnected => {
							return Result::Err(());
						}
					}
				}
			}
		}
	}
}
