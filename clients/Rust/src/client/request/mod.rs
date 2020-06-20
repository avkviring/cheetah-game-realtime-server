use std::sync::mpsc::{Receiver, TryRecvError};

use crate::client::Client;
use crate::client::command::C2SCommandUnion;

pub enum ClientRequestType {
	SendCommandToServer(C2SCommandUnion),
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
		let result = self.receiver.try_recv();
		match result {
			Ok(request) => {
				match request {
					ClientRequestType::SendCommandToServer(command) => {
						client.schedule_command_to_server(command);
						Result::Ok(RequestResult::Ok)
					}
					ClientRequestType::Close => {
						Result::Ok(RequestResult::Close)
					}
				}
			}
			Err(e) => {
				match e {
					TryRecvError::Empty => {
						Result::Ok(RequestResult::Ok)
					}
					TryRecvError::Disconnected => {
						Result::Err(())
					}
				}
			}
		}
	}
}
