use std::sync::mpsc::Sender;

use crate::relay::client::s2ccommand::S2CCommandUnion;

pub struct ClientController {
	pub sender: Sender<ClientRequestType>
}

pub enum ClientRequestType {
	GetS2CCommands(Sender<Vec<S2CCommandUnion>>)
}


impl ClientController {
	pub fn new(sender: Sender<ClientRequestType>) -> ClientController {
		ClientController {
			sender
		}
	}
}

