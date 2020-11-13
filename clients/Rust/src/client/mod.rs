use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use cheetah_relay_common::protocol::frame::applications::ApplicationCommandDescription;
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::{ConnectionStatus, UdpClient};

use crate::clients::ClientRequest;

pub mod ffi;
pub mod command;


pub struct Client {
	state: Arc<Mutex<ConnectionStatus>>,
	out_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	udp_client: UdpClient,
	receiver: Receiver<ClientRequest>,
}


impl Client {
	pub fn new(
		server_address: SocketAddr,
		user_public_key: UserPublicKey,
		user_private_key: UserPrivateKey,
		out_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
		in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
		state: Arc<Mutex<ConnectionStatus>>,
		receiver: Receiver<ClientRequest>,
	) -> Result<Client, ()> {
		Result::Ok(
			Client {
				state,
				out_commands,
				in_commands,
				udp_client: UdpClient::new(user_private_key, user_public_key, server_address)?,
				receiver,
			})
	}
	
	
	pub fn run(mut self) {
		loop {
			let cloned_out_commands = self.out_commands.clone();
			let out_commands = &mut cloned_out_commands.lock().unwrap();
			while let Some(command) = out_commands.pop_back() {
				self.udp_client.protocol.out_commands_collector.add_command(command);
			}
			
			let in_commands_from_protocol = self.udp_client.protocol.in_commands_collector.get_commands();
			let cloned_in_commands = self.in_commands.clone();
			let in_commands = &mut cloned_in_commands.lock().unwrap();
			while let Some(command) = in_commands_from_protocol.pop_back() {
				in_commands.push_front(command);
			}
			
			self.udp_client.cycle(&Instant::now());
			
			let state = self.state.clone();
			*state.lock().unwrap() = self.udp_client.state.clone();
			
			
			match self.receiver.try_recv() {
				Ok(ClientRequest::Close) => {
					return;
				}
				Err(_) => {}
			}
			
			
			thread::sleep(Duration::from_millis(1));
		}
	}
}