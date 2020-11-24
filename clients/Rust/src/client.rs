use std::collections::VecDeque;
use std::net::SocketAddr;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use cheetah_relay_common::commands::command::C2SCommandWithMeta;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType, ApplicationCommandDescription};
use cheetah_relay_common::room::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::client::{ConnectionStatus, UdpClient};

use crate::registry::ClientRequest;

#[derive(Debug)]
pub struct Client {
	state: Arc<Mutex<ConnectionStatus>>,
	out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
	in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	udp_client: UdpClient,
	receiver: Receiver<ClientRequest>,
	protocol_time_offset: Option<Duration>,
}

#[derive(Debug)]
pub struct OutApplicationCommand {
	pub channel_type: ApplicationCommandChannelType,
	pub command: C2SCommandWithMeta,
}


impl Client {
	pub fn new(
		server_address: SocketAddr,
		user_public_key: UserPublicKey,
		user_private_key: UserPrivateKey,
		out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
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
				protocol_time_offset: None,
			})
	}
	
	
	pub fn run(mut self) {
		loop {
			let arc_out_commands = self.out_commands.clone();
			let lock_for_out_commands = arc_out_commands.lock();
			let mut out_commands = lock_for_out_commands.unwrap();
			while let Some(command) = out_commands.pop_back() {
				self
					.udp_client
					.protocol
					.out_commands_collector
					.add_command(command.channel_type, ApplicationCommand::C2SCommandWithMeta(command.command));
			}
			drop(out_commands);
			drop(arc_out_commands);
			
			let in_commands_from_protocol = self.udp_client.protocol.in_commands_collector.get_commands();
			let arc_in_commands = self.in_commands.clone();
			let mut in_commands = arc_in_commands.lock().unwrap();
			while let Some(command) = in_commands_from_protocol.pop_back() {
				in_commands.push_front(command);
			}
			
			drop(in_commands);
			drop(arc_in_commands);
			
			let mut now = Instant::now();
			if let Some(offset) = self.protocol_time_offset {
				now = now.add(offset);
			}
			self.udp_client.cycle(&now);
			
			let arc_state = self.state.clone();
			*arc_state.lock().unwrap() = self.udp_client.state.clone();
			drop(arc_state);
			
			match self.receiver.try_recv() {
				Ok(ClientRequest::Close) => {
					self.udp_client.protocol.disconnect_handler.disconnect();
					let now = Instant::now();
					self.udp_client.cycle(&now);
					return;
				}
				Ok(ClientRequest::SetProtocolTimeOffset(duration)) => {
					self.protocol_time_offset = Option::Some(duration);
				}
				Err(_) => {}
			}
			
			thread::sleep(Duration::from_millis(7));
		}
	}
}