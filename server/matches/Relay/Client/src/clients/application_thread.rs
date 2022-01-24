use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::load::CreateGameObjectCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::network::client::ConnectionStatus;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use cheetah_matches_relay_common::protocol::frame::channel::ChannelType;
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::clients::network_thread::C2SCommandWithChannel;
use crate::clients::{ClientRequest, SharedClientStatistics};
use crate::ffi::channel::Channel;
use crate::ffi::{BufferFFI, GameObjectIdFFI};

///
/// Взаимодействие с сетевым потоком клиента, через Sender
///
pub struct ApplicationThreadClient {
	user_id: RoomMemberId,
	commands_from_server: Receiver<CommandWithChannel>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	request_to_client: Sender<ClientRequest>,
	channel: ChannelType,
	game_object_id_generator: u32,
	pub shared_statistics: SharedClientStatistics,
	pub listener_long_value: Option<extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, i64)>,
	pub listener_float_value: Option<extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, f64)>,
	pub listener_event: Option<extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI)>,
	pub listener_structure: Option<extern "C" fn(RoomMemberId, &GameObjectIdFFI, FieldId, &BufferFFI)>,
	pub listener_create_object: Option<extern "C" fn(&GameObjectIdFFI, u16)>,
	pub listener_delete_object: Option<extern "C" fn(&GameObjectIdFFI)>,
	pub listener_created_object: Option<extern "C" fn(&GameObjectIdFFI)>,
}

impl Drop for ApplicationThreadClient {
	fn drop(&mut self) {
		if self.request_to_client.send(ClientRequest::Close).is_ok() {
			self.handler.take().unwrap().join().unwrap();
		}
	}
}

impl ApplicationThreadClient {
	pub fn new(
		user_id: RoomMemberId,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Receiver<CommandWithChannel>,
		sender: Sender<ClientRequest>,
		shared_statistics: SharedClientStatistics,
	) -> Self {
		Self {
			user_id,
			commands_from_server: in_commands,
			handler: Option::Some(handler),
			state,
			request_to_client: sender,
			channel: ChannelType::ReliableSequenceByGroup(0),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			shared_statistics,
			listener_long_value: None,
			listener_float_value: None,
			listener_event: None,
			listener_structure: None,
			listener_delete_object: None,
			listener_create_object: None,
			listener_created_object: None,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::SetProtocolTimeOffset(time_offset))
	}

	pub fn send(&mut self, command: C2SCommand) -> Result<(), SendError<ClientRequest>> {
		let out_command = C2SCommandWithChannel {
			channel_type: self.channel.clone(),
			command,
		};
		self.request_to_client.send(ClientRequest::SendCommandToServer(out_command))
	}

	pub fn get_connection_status(&self, result: &mut ConnectionStatus) -> Result<(), PoisonError<MutexGuard<ConnectionStatus>>> {
		*result = *self.state.lock()?;
		Ok(())
	}

	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroup) {
		self.channel = match channel {
			Channel::ReliableUnordered => ChannelType::ReliableUnordered,
			Channel::UnreliableUnordered => ChannelType::UnreliableUnordered,
			Channel::ReliableOrderedByObject => ChannelType::ReliableOrderedByObject,
			Channel::UnreliableOrderedByObject => ChannelType::UnreliableOrderedByObject,
			Channel::ReliableOrderedByGroup => ChannelType::ReliableOrderedByGroup(group),
			Channel::UnreliableOrderedByGroup => ChannelType::UnreliableOrderedByGroup(group),
			Channel::ReliableSequenceByObject => ChannelType::ReliableSequenceByObject,
			Channel::ReliableSequenceByGroup => ChannelType::ReliableSequenceByGroup(group),
		}
	}

	pub fn receive(&mut self) {
		while let Ok(command) = self.commands_from_server.try_recv() {
			if let BothDirectionCommand::S2CWithCreator(command_with_user) = command.both_direction_command {
				match command_with_user.command {
					S2CCommand::Create(command) => {
						if let Some(ref listener) = self.listener_create_object {
							let object_id = From::from(&command.object_id);
							listener(&object_id, command.template);
						}
					}
					S2CCommand::Created(command) => {
						if let Some(ref listener) = self.listener_created_object {
							let object_id = From::from(&command.object_id);
							listener(&object_id);
						}
					}
					S2CCommand::SetLong(command) => {
						if let Some(ref listener) = self.listener_long_value {
							let object_id = From::from(&command.object_id);
							listener(command_with_user.creator, &object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetDouble(command) => {
						if let Some(ref listener) = self.listener_float_value {
							let object_id = From::from(&command.object_id);
							listener(command_with_user.creator, &object_id, command.field_id, command.value);
						}
					}
					S2CCommand::SetStructure(command) => {
						if let Some(ref listener) = self.listener_structure {
							let object_id = From::from(&command.object_id);
							listener(
								command_with_user.creator,
								&object_id,
								command.field_id,
								&From::from(&command.structure),
							);
						}
					}
					S2CCommand::Event(command) => {
						if let Some(ref listener) = self.listener_event {
							let object_id: GameObjectIdFFI = From::from(&command.object_id);
							listener(
								command_with_user.creator,
								&object_id,
								command.field_id,
								&From::from(&command.event),
							);
						}
					}
					S2CCommand::Delete(command) => {
						if let Some(ref listener) = self.listener_delete_object {
							let object_id = From::from(&command.object_id);
							listener(&object_id);
						}
					}
				}
			}
		}
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> Result<GameObjectIdFFI, SendError<ClientRequest>> {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, GameObjectOwner::User(self.user_id));
		self.send(C2SCommand::Create(CreateGameObjectCommand {
			object_id: game_object_id.clone(),
			template,
			access_groups: AccessGroups(access_group),
		}))?;

		Ok(From::from(&game_object_id))
	}

	pub fn set_rtt_emulation(&mut self, rtt: Duration, rtt_dispersion: f64) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client
			.send(ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion))
	}

	pub fn set_drop_emulation(&mut self, drop_probability: f64, drop_time: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client
			.send(ClientRequest::ConfigureDropEmulation(drop_probability, drop_time))
	}

	pub fn reset_emulation(&mut self) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ResetEmulation)
	}

	pub fn attach_to_room(&mut self) -> Result<(), SendError<ClientRequest>> {
		// удаляем все пришедшие команды (ситуация возникает при attach/detach)
		while self.commands_from_server.try_recv().is_ok() {}
		self.send(C2SCommand::AttachToRoom)
	}
}
