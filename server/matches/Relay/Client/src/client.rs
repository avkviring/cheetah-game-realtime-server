use std::net::SocketAddr;
use std::ops::Add;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::network::client::{ConnectionStatus, NetworkClient};
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_matches_relay_common::protocol::frame::channel::ApplicationCommandChannelType;
use cheetah_matches_relay_common::protocol::others::rtt::RoundTripTime;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::registry::ClientRequest;

#[derive(Debug)]
pub struct Client {
	state: Arc<Mutex<ConnectionStatus>>,
	commands_from_server: Sender<CommandWithChannel>,
	udp_client: NetworkClient,
	request_from_controller: Receiver<ClientRequest>,
	protocol_time_offset: Option<Duration>,
	current_frame_id: Arc<AtomicU64>,
	rtt_in_ms: Arc<AtomicU64>,
	average_retransmit_frames: Arc<AtomicU32>,
	running: bool,
}

#[derive(Debug)]
pub struct C2SCommandWithChannel {
	pub channel_type: ApplicationCommandChannelType,
	pub command: C2SCommand,
}

impl Client {
	pub fn new(
		server_address: SocketAddr,
		member_id: RoomMemberId,
		room_id: RoomId,
		user_private_key: UserPrivateKey,
		in_commands: Sender<CommandWithChannel>,
		state: Arc<Mutex<ConnectionStatus>>,
		receiver: Receiver<ClientRequest>,
		current_frame_id: Arc<AtomicU64>,
		rtt_in_ms: Arc<AtomicU64>,
		average_retransmit_frames: Arc<AtomicU32>,
	) -> Result<Client, ()> {
		Result::Ok(Client {
			state,
			commands_from_server: in_commands,
			udp_client: NetworkClient::new(
				false,
				user_private_key,
				member_id,
				room_id,
				server_address,
				current_frame_id.load(Ordering::Relaxed),
			)?,
			request_from_controller: receiver,
			protocol_time_offset: None,
			current_frame_id,
			rtt_in_ms,
			average_retransmit_frames,
			running: false,
		})
	}

	pub fn run(mut self) {
		self.running = true;
		while self.running {
			let now = self.get_now_time();
			self.udp_client.cycle(&now);
			self.commands_from_server();
			self.request_from_controller();
			self.update_state();
			thread::sleep(Duration::from_millis(7));
		}
	}

	///
	/// Текущее время, с учетом коррекции для тестов
	///
	fn get_now_time(&mut self) -> Instant {
		let now = Instant::now();
		if let Some(offset) = self.protocol_time_offset {
			now.add(offset)
		} else {
			now
		}
	}

	///
	/// Обработка команд с сервера
	///
	fn commands_from_server(&mut self) {
		let in_commands_from_protocol = self.udp_client.protocol.in_commands_collector.get_commands();
		while let Some(command) = in_commands_from_protocol.pop_back() {
			match self.commands_from_server.send(command) {
				Ok(_) => {}
				Err(e) => {
					self.running = false;
					log::error!("[client] error send command from server {:?}", e)
				}
			}
		}
	}

	///
	/// Обработка команд из контроллера
	///
	fn request_from_controller(&mut self) {
		while let Result::Ok(command) = self.request_from_controller.try_recv() {
			match command {
				ClientRequest::Close => {
					self.udp_client.protocol.disconnect_handler.disconnect();
					let now = Instant::now();
					self.udp_client.cycle(&now);
					self.running = false;
				}
				ClientRequest::SetProtocolTimeOffset(duration) => {
					self.protocol_time_offset = Option::Some(duration);
				}
				ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion) => {
					self.udp_client.channel.config_emulator(|emulator| {
						emulator.configure_rtt(rtt, rtt_dispersion);
					})
				}
				ClientRequest::ConfigureDropEmulation(drop_probability, drop_time) => {
					self.udp_client.channel.config_emulator(|emulator| {
						emulator.configure_drop(drop_probability, drop_time);
					})
				}
				ClientRequest::ResetEmulation => {
					self.udp_client.channel.reset_emulator();
				}
				ClientRequest::SendCommandToServer(command) => {
					self.udp_client
						.protocol
						.out_commands_collector
						.add_command(command.channel_type, BothDirectionCommand::C2S(command.command));
				}
			}
		}
	}

	///
	/// Обновление статистики для контроллера
	///
	fn update_state(&mut self) {
		self.current_frame_id
			.store(self.udp_client.protocol.next_frame_id, Ordering::Relaxed);
		self.rtt_in_ms.store(
			self.udp_client
				.protocol
				.rtt
				.get_rtt()
				.unwrap_or(Duration::from_millis(0))
				.as_millis() as u64,
			Ordering::Relaxed,
		);
		self.average_retransmit_frames.store(
			self.udp_client
				.protocol
				.retransmitter
				.statistics
				.get_average_retransmit_frames(&Instant::now())
				.unwrap_or(0) as u32,
			Ordering::Relaxed,
		);

		let arc_state = self.state.clone();
		*arc_state.lock().unwrap() = self.udp_client.state.clone();
		drop(arc_state);
	}
}
