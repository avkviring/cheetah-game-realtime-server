use std::collections::VecDeque;
use std::ops::Sub;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType, ApplicationCommandDescription, ChannelGroupId};
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::client::OutApplicationCommand;
use crate::ffi::channel::Channel;
use crate::ffi::GameObjectIdFFI;
use crate::registry::ClientRequest;

///
/// Управление сетевым потоком клиента
///
#[derive(Debug)]
pub struct ClientController {
	out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
	in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	sender: Sender<ClientRequest>,
	create_time: Instant,
	channel: ApplicationCommandChannelType,
	listener_long_value: Option<extern fn(GameObjectIdFFI, FieldID, i64)>,
}


impl Drop for ClientController {
	fn drop(&mut self) {
		match self.sender.send(ClientRequest::Close) {
			Ok(_) => {
				self.handler.take().unwrap().join().unwrap();
			}
			Err(_) => {}
		}
	}
}


impl ClientController {
	pub fn new(
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
		out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
		sender: Sender<ClientRequest>,
	) -> Self {
		Self {
			out_commands,
			in_commands,
			handler: Option::Some(handler),
			state,
			sender,
			create_time: Instant::now(),
			channel: ApplicationCommandChannelType::ReliableSequenceByGroup(0),
			listener_long_value: None,
		}
	}
	
	
	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) {
		self.sender.send(ClientRequest::SetProtocolTimeOffset(time_offset)).unwrap();
	}
	
	pub fn send(&mut self, command: C2SCommand) {
		let meta = C2SMetaCommandInformation { timestamp: Instant::now().sub(self.create_time).as_millis() as u64 };
		let command = OutApplicationCommand {
			channel_type: self.channel.clone(),
			command: C2SCommandWithMeta {
				meta,
				command,
			},
		};
		self.out_commands.lock().unwrap().push_front(command);
	}
	
	pub fn get_connection_status(&self) -> ConnectionStatus {
		*self.state.lock().unwrap()
	}
	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroupId) {
		self.channel = match channel {
			Channel::ReliableUnordered => {
				ApplicationCommandChannelType::ReliableUnordered
			}
			Channel::UnreliableUnordered => {
				ApplicationCommandChannelType::UnreliableUnordered
			}
			Channel::ReliableOrderedByObject => {
				ApplicationCommandChannelType::ReliableOrderedByObject
			}
			Channel::UnreliableOrderedByObject => {
				ApplicationCommandChannelType::UnreliableOrderedByObject
			}
			Channel::ReliableOrderedByGroup => {
				ApplicationCommandChannelType::ReliableOrderedByGroup(group)
			}
			Channel::UnreliableOrderedByGroup => {
				ApplicationCommandChannelType::UnreliableOrderedByGroup(group)
			}
			Channel::ReliableSequenceByObject => {
				ApplicationCommandChannelType::ReliableSequenceByObject
			}
			Channel::ReliableSequenceByGroup => {
				ApplicationCommandChannelType::ReliableSequenceByGroup(group)
			}
		}
	}
	
	pub fn process_in_commands(&mut self) {
		let commands = self.in_commands.clone();
		let mut commands = commands.lock().unwrap();
		let commands = &mut *commands;
		let cloned_commands = commands.clone();
		commands.clear();
		
		for command in cloned_commands {
			if let ApplicationCommand::S2CCommandWithMeta(command) = command.command {
				match command.command {
					S2CCommand::Create(_) => {}
					S2CCommand::SetLong(command) => {
						if let Some(ref listener) = self.listener_long_value {
							let object_id = From::from(&command.object_id);
							listener(object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetFloat64(_) => {}
					S2CCommand::SetStruct(_) => {}
					S2CCommand::Event(_) => {}
					S2CCommand::Delete(_) => {}
				}
			}
		}
	}
	
	
	pub fn register_long_value_listener(&mut self, listener: extern fn(GameObjectIdFFI, FieldID, i64)) {
		self.listener_long_value = Option::Some(listener);
	}
}
