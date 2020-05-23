use std::sync::mpsc::Receiver;

use crate::relay::client::config::ClientConfig;
use crate::relay::client::controller::ClientRequestType;

pub mod controller;
pub mod config;
pub mod s2ccommand;


pub struct Client {
	receiver: Receiver<ClientRequestType>
}


impl Client {
	pub fn new(client_config: ClientConfig, receiver: Receiver<ClientRequestType>) -> Client {
		Client {
			receiver
		}
	}
	
	pub fn run(&mut self) {
		
	}
}