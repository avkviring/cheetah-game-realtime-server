use std::collections::VecDeque;
use std::ops::Sub;
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cheetah_relay_common::commands::command::load::CreatingGameObjectCommand;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::protocol::frame::applications::{
	ApplicationCommand, ApplicationCommandChannelType, ApplicationCommandDescription, ChannelGroupId,
};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::client::OutApplicationCommand;
use crate::ffi::channel::Channel;
use crate::ffi::{BufferFFI, GameObjectIdFFI};
use crate::registry::ClientRequest;

///
/// Управление сетевым потоком клиента
///
pub struct ClientController {
	user_public_key: UserPublicKey,
	out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
	in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	sender: Sender<ClientRequest>,
	create_time: Instant,
	channel: ApplicationCommandChannelType,
	game_object_id_generator: u32,
	pub current_frame_id: Arc<AtomicU64>,
	listener_long_value: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, i64)>,
	listener_float_value: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, f64)>,
	listener_event: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, &BufferFFI)>,
	listener_structure: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, &BufferFFI)>,
	listener_delete_object: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI)>,
	listener_create_object: Option<extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, u16)>,
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
		user_public_key: UserPublicKey,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Arc<Mutex<VecDeque<ApplicationCommandDescription>>>,
		out_commands: Arc<Mutex<VecDeque<OutApplicationCommand>>>,
		sender: Sender<ClientRequest>,
		current_frame_id: Arc<AtomicU64>,
	) -> Self {
		Self {
			user_public_key,
			out_commands,
			in_commands,
			handler: Option::Some(handler),
			state,
			sender,
			create_time: Instant::now(),
			channel: ApplicationCommandChannelType::ReliableSequenceByGroup(0),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			current_frame_id,
			listener_long_value: None,
			listener_float_value: None,
			listener_event: None,
			listener_structure: None,
			listener_delete_object: None,
			listener_create_object: None,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) {
		self.sender.send(ClientRequest::SetProtocolTimeOffset(time_offset)).unwrap();
	}

	pub fn send(&mut self, command: C2SCommand) {
		let meta = C2SMetaCommandInformation {
			timestamp: Instant::now().sub(self.create_time).as_millis() as u64,
		};
		let command = OutApplicationCommand {
			channel_type: self.channel.clone(),
			command: C2SCommandWithMeta { meta, command },
		};
		self.out_commands.lock().unwrap().push_front(command);
	}

	pub fn get_connection_status(&self) -> ConnectionStatus {
		*self.state.lock().unwrap()
	}

	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroupId) {
		self.channel = match channel {
			Channel::ReliableUnordered => ApplicationCommandChannelType::ReliableUnordered,
			Channel::UnreliableUnordered => ApplicationCommandChannelType::UnreliableUnordered,
			Channel::ReliableOrderedByObject => ApplicationCommandChannelType::ReliableOrderedByObject,
			Channel::UnreliableOrderedByObject => ApplicationCommandChannelType::UnreliableOrderedByObject,
			Channel::ReliableOrderedByGroup => ApplicationCommandChannelType::ReliableOrderedByGroup(group),
			Channel::UnreliableOrderedByGroup => ApplicationCommandChannelType::UnreliableOrderedByGroup(group),
			Channel::ReliableSequenceByObject => ApplicationCommandChannelType::ReliableSequenceByObject,
			Channel::ReliableSequenceByGroup => ApplicationCommandChannelType::ReliableSequenceByGroup(group),
		}
	}

	pub fn receive(&mut self) {
		let commands_arc = self.in_commands.clone();
		let commands_lock = commands_arc.lock();
		let mut commands = commands_lock.unwrap();
		let mut cloned_commands = commands.clone();
		commands.clear();
		drop(commands);

		while let Some(command) = cloned_commands.pop_back() {
			if let ApplicationCommand::S2CCommandWithMeta(command) = command.command {
				let meta = &command.meta;
				match command.command {
					S2CCommand::Create(command) => {
						if let Some(ref listener) = self.listener_create_object {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.template);
						}
					}
					S2CCommand::SetLong(command) => {
						if let Some(ref listener) = self.listener_long_value {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetFloat64(command) => {
						if let Some(ref listener) = self.listener_float_value {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetStruct(command) => {
						if let Some(ref listener) = self.listener_structure {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, &From::from(&command.structure));
						}
					}
					S2CCommand::Event(command) => {
						if let Some(ref listener) = self.listener_event {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, &From::from(&command.event));
						}
					}
					S2CCommand::Delete(command) => {
						if let Some(ref listener) = self.listener_delete_object {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id);
						}
					}
					S2CCommand::Created(_) => {
						todo!();
					}
				}
			}
		}
	}

	pub fn register_long_value_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, i64)) {
		self.listener_long_value = Option::Some(listener);
	}
	pub fn register_float_value_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, f64)) {
		self.listener_float_value = Option::Some(listener);
	}
	pub fn register_event_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, &BufferFFI)) {
		self.listener_event = Option::Some(listener);
	}
	pub fn register_structure_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, FieldID, &BufferFFI)) {
		self.listener_structure = Option::Some(listener);
	}
	pub fn register_delete_object_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI)) {
		self.listener_delete_object = Option::Some(listener);
	}

	pub fn register_create_object_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformation, &GameObjectIdFFI, u16)) {
		self.listener_create_object = Option::Some(listener);
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> GameObjectIdFFI {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, ObjectOwner::User(self.user_public_key));
		self.send(C2SCommand::Create(CreatingGameObjectCommand {
			object_id: game_object_id.clone(),
			template,
			access_groups: AccessGroups(access_group),
		}));

		From::from(&game_object_id)
	}
}
