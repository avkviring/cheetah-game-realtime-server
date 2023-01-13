use std::slice;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::create::CreateGameObjectCommand;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::network::client::ConnectionStatus;
use cheetah_common::protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use cheetah_common::protocol::frame::channel::ChannelType;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomMemberId;

use crate::clients::network_thread::C2SCommandWithChannel;
use crate::clients::{ClientRequest, SharedClientStatistics};
use crate::ffi::channel::Channel;
use crate::ffi::command::S2CCommandFFI;

///
/// Взаимодействие с сетевым потоком клиента, через Sender
///
pub struct ApplicationThreadClient {
	member_id: RoomMemberId,
	s2c_receiver: Receiver<CommandWithChannel>,
	handler: Option<JoinHandle<()>>,
	state: Arc<Mutex<ConnectionStatus>>,
	server_time: Arc<Mutex<Option<u64>>>,
	request_to_client: Sender<ClientRequest>,
	channel: ChannelType,
	game_object_id_generator: u32,
	pub shared_statistics: SharedClientStatistics,
}

impl Drop for ApplicationThreadClient {
	fn drop(&mut self) {
		if self.request_to_client.send(ClientRequest::Close(DisconnectByCommandReason::ClientStopped)).is_ok() {
			self.handler.take().unwrap().join().unwrap();
		}
	}
}

impl ApplicationThreadClient {
	pub fn new(
		member_id: RoomMemberId,
		handler: JoinHandle<()>,
		state: Arc<Mutex<ConnectionStatus>>,
		in_commands: Receiver<CommandWithChannel>,
		sender: Sender<ClientRequest>,
		shared_statistics: SharedClientStatistics,
		server_time: Arc<Mutex<Option<u64>>>,
	) -> Self {
		Self {
			member_id,
			s2c_receiver: in_commands,
			handler: Some(handler),
			state,
			server_time,
			request_to_client: sender,
			channel: ChannelType::ReliableSequence(ChannelGroup(0)),
			game_object_id_generator: GameObjectId::CLIENT_OBJECT_ID_OFFSET,
			shared_statistics,
		}
	}

	pub fn set_protocol_time_offset(&mut self, time_offset: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::SetProtocolTimeOffsetForTest(time_offset))
	}

	pub fn send(&mut self, command: C2SCommand) -> Result<(), SendError<ClientRequest>> {
		let out_command = C2SCommandWithChannel { channel_type: self.channel, command };
		tracing::info!("{:?}", out_command);
		self.request_to_client.send(ClientRequest::SendCommandToServer(out_command))
	}

	pub fn get_connection_status(&self) -> Result<ConnectionStatus, PoisonError<MutexGuard<'_, ConnectionStatus>>> {
		Ok(self.state.lock()?.clone())
	}

	#[allow(clippy::unwrap_in_result)]
	pub fn get_server_time(&self) -> Option<u64> {
		*self.server_time.lock().unwrap()
	}

	pub fn set_current_channel(&mut self, channel: Channel, group: ChannelGroup) {
		self.channel = match channel {
			Channel::ReliableUnordered => ChannelType::ReliableUnordered,
			Channel::UnreliableUnordered => ChannelType::UnreliableUnordered,
			Channel::ReliableOrdered => ChannelType::ReliableOrdered(group),
			Channel::UnreliableOrdered => ChannelType::UnreliableOrdered(group),
			Channel::ReliableSequence => ChannelType::ReliableSequence(group),
		}
	}

	pub unsafe fn receive(&mut self, commands: *mut S2CCommandFFI, count: &mut u16) {
		*count = 0;
		let commands: &mut [S2CCommandFFI] = slice::from_raw_parts_mut(commands, 1024);

		while let Ok(command) = self.s2c_receiver.try_recv() {
			if let BothDirectionCommand::S2CWithCreator(member_with_creator) = command.both_direction_command {
				let mut command_ffi = &mut commands[*count as usize];
				match member_with_creator.command {
					S2CCommand::Create(command) => {
						command_ffi.command_type = CommandTypeId::CreateGameObject;
						command_ffi.command.create = command;
					}
					S2CCommand::Created(command) => {
						command_ffi.command_type = CommandTypeId::CreatedGameObject;
						command_ffi.command.created = command;
					}

					S2CCommand::SetLong(command) => {
						command_ffi.command_type = CommandTypeId::SetLong;
						command_ffi.command.set_long = command;
					}
					S2CCommand::SetDouble(command) => {
						command_ffi.command_type = CommandTypeId::SetDouble;
						command_ffi.command.set_double = command;
					}
					S2CCommand::SetStructure(command) => {
						command_ffi.command_type = CommandTypeId::SetStructure;
						command_ffi.command.set_structure = command;
					}

					S2CCommand::Event(command) => {
						command_ffi.command_type = CommandTypeId::SendEvent;
						command_ffi.command.event = command;
					}
					S2CCommand::Delete(command) => {
						command_ffi.command_type = CommandTypeId::DeleteObject;
						command_ffi.command.delete = command;
					}
					S2CCommand::DeleteField(command) => {
						command_ffi.command_type = CommandTypeId::DeleteField;
						command_ffi.command.delete_field = command;
					}
					S2CCommand::Forwarded(_s) => {}
					S2CCommand::MemberConnected(command) => {
						command_ffi.command_type = CommandTypeId::MemberConnected;
						command_ffi.command.member_connect = command;
					}
					S2CCommand::MemberDisconnected(command) => {
						command_ffi.command_type = CommandTypeId::MemberDisconnected;
						command_ffi.command.member_disconnect = command;
					}
				}
				if *count == 1023 {
					break;
				}
				*count += 1;
			}
		}
	}

	pub fn create_game_object(&mut self, template: u16, access_group: u64) -> Result<GameObjectId, SendError<ClientRequest>> {
		self.game_object_id_generator += 1;
		let game_object_id = GameObjectId::new(self.game_object_id_generator, GameObjectOwner::Member(self.member_id));
		self.send(C2SCommand::CreateGameObject(CreateGameObjectCommand {
			object_id: game_object_id,
			template,
			access_groups: AccessGroups(access_group),
		}))?;

		Ok(game_object_id)
	}

	pub fn set_rtt_emulation(&mut self, rtt: Duration, rtt_dispersion: f64) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion))
	}

	pub fn set_drop_emulation(&mut self, drop_probability: f64, drop_time: Duration) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ConfigureDropEmulation(drop_probability, drop_time))
	}

	pub fn reset_emulation(&mut self) -> Result<(), SendError<ClientRequest>> {
		self.request_to_client.send(ClientRequest::ResetEmulation)
	}

	pub fn attach_to_room(&mut self) -> Result<(), SendError<ClientRequest>> {
		// удаляем все пришедшие команды (ситуация возникает при attach/detach)
		while self.s2c_receiver.try_recv().is_ok() {}
		self.send(C2SCommand::AttachToRoom)
	}
}
