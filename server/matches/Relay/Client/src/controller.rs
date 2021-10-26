use std::ops::Sub;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cheetah_matches_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_matches_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_matches_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::network::client::ConnectionStatus;
use cheetah_matches_relay_common::protocol::frame::applications::{
	ApplicationCommand, ApplicationCommandChannelType, ApplicationCommandDescription, ChannelGroupId,
};
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::UserId;

use crate::client::OutApplicationCommand;
use crate::ffi::channel::Channel;
use crate::ffi::command::S2CMetaCommandInformationFFI;
use crate::ffi::{BufferFFI, GameObjectIdFFI};
use crate::registry::ClientRequest;

///
/// Управление сетевым потоком клиента
///
pub struct ClientController {
	user_id: UserId,
	commands_from_server: Receiver<ApplicationCommandDescription>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	request_to_client: Sender<ClientRequest>,
	create_time: Instant,
	channel: ApplicationCommandChannelType,
	game_object_id_generator: u32,
	pub source_object: Option<GameObjectId>,
	pub current_frame_id: Arc<AtomicU64>,
	pub rtt_in_ms: Arc<AtomicU64>,
	pub average_retransmit_frames: Arc<AtomicU32>,
	listener_long_value: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, i64)>,
	listener_float_value: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, f64)>,
	listener_event: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)>,
	listener_structure: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)>,
	listener_delete_object: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI)>,
	listener_create_object: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, u16)>,
	pub listener_created_object: Option<extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI)>,
	pub error_in_client_thread: bool,
}

impl Drop for ClientController {
	fn drop(&mut self) {
		match self.request_to_client.send(ClientRequest::Close) {
			Ok(_) => {
				self.handler.take().unwrap().join().unwrap();
			}
			Err(_) => {}
		}
	}
}

impl ClientController {
	pub fn new(
		user_id: UserId,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Receiver<ApplicationCommandDescription>,
		sender: Sender<ClientRequest>,
		current_frame_id: Arc<AtomicU64>,
		rtt_in_ms: Arc<AtomicU64>,
		average_retransmit_frames: Arc<AtomicU32>,
	) -> Self {
		Self {
			user_id,
			commands_from_server: in_commands,
			handler: Option::Some(handler),
			state,
			request_to_client: sender,
			create_time: Instant::now(),
			channel: ApplicationCommandChannelType::ReliableSequenceByGroup(0),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			source_object: None,
			current_frame_id,
			rtt_in_ms,
			average_retransmit_frames,
			listener_long_value: None,
			listener_float_value: None,
			listener_event: None,
			listener_structure: None,
			listener_delete_object: None,
			listener_create_object: None,
			listener_created_object: None,
			error_in_client_thread: false,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) {
		self.request_to_client.send(ClientRequest::SetProtocolTimeOffset(time_offset)).unwrap();
	}

	pub fn send(&mut self, command: C2SCommand) {
		let meta = C2SMetaCommandInformation {
			timestamp: Instant::now().sub(self.create_time).as_millis() as u64,
			source_object: self.source_object.clone(),
		};
		let command = OutApplicationCommand {
			channel_type: self.channel.clone(),
			command: C2SCommandWithMeta { meta, command },
		};
		match self.request_to_client.send(ClientRequest::SendCommandToServer(command)) {
			Ok(_) => {}
			Err(e) => {
				log::error!("[controller] error send to channel {:?}", e);
				self.error_in_client_thread = true;
			}
		}
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
		while let Ok(command) = self.commands_from_server.try_recv() {
			if let ApplicationCommand::S2CCommandWithMeta(command) = command.command {
				let meta = &S2CMetaCommandInformationFFI::from(&command.meta);
				match command.command {
					S2CCommand::Create(command) => {
						if let Some(ref listener) = self.listener_create_object {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.template);
						}
					}
					S2CCommand::Created(command) => {
						if let Some(ref listener) = self.listener_created_object {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id);
						}
					}
					S2CCommand::SetLong(command) => {
						if let Some(ref listener) = self.listener_long_value {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetFloat(command) => {
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
							let object_id: GameObjectIdFFI = From::from(&command.object_id);
							listener(meta, &object_id, command.field_id, &From::from(&command.event));
						}
					}
					S2CCommand::Delete(command) => {
						if let Some(ref listener) = self.listener_delete_object {
							let object_id = From::from(&command.object_id);
							listener(meta, &object_id);
						}
					}
				}
			}
		}
	}

	pub fn register_long_value_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, i64)) {
		self.listener_long_value = Option::Some(listener);
	}
	pub fn register_float_value_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, f64)) {
		self.listener_float_value = Option::Some(listener);
	}
	pub fn register_event_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)) {
		self.listener_event = Option::Some(listener);
	}
	pub fn register_structure_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, FieldId, &BufferFFI)) {
		self.listener_structure = Option::Some(listener);
	}
	pub fn register_delete_object_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI)) {
		self.listener_delete_object = Option::Some(listener);
	}

	pub fn register_create_object_listener(&mut self, listener: extern "C" fn(&S2CMetaCommandInformationFFI, &GameObjectIdFFI, u16)) {
		self.listener_create_object = Option::Some(listener);
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> GameObjectIdFFI {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, GameObjectOwner::User(self.user_id));
		self.send(C2SCommand::Create(CreateGameObjectCommand {
			object_id: game_object_id.clone(),
			template,
			access_groups: AccessGroups(access_group),
		}));

		From::from(&game_object_id)
	}

	pub fn set_rtt_emulation(&mut self, rtt: Duration, rtt_dispersion: f64) {
		match self.request_to_client.send(ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion)) {
			Ok(_) => {}
			Err(e) => {
				log::error!("[controller] error send to channel {:?}", e);
				self.error_in_client_thread = true;
			}
		}
	}

	pub fn set_drop_emulation(&mut self, drop_probability: f64, drop_time: Duration) {
		match self
			.request_to_client
			.send(ClientRequest::ConfigureDropEmulation(drop_probability, drop_time))
		{
			Ok(_) => {}
			Err(e) => {
				log::error!("[controller] error send to channel {:?}", e);
				self.error_in_client_thread = true;
			}
		}
	}

	pub fn reset_emulation(&mut self) {
		match self.request_to_client.send(ClientRequest::ResetEmulation) {
			Ok(_) => {}
			Err(e) => {
				log::error!("[controller] error send to channel {:?}", e);
				self.error_in_client_thread = true;
			}
		}
	}

	pub fn attach_to_room(&mut self) {
		// удаляем все пришедшие команды (ситуация возникает при attach/detach)
		while let Ok(_) = self.commands_from_server.try_recv() {}
		self.send(C2SCommand::AttachToRoom);
	}
}
