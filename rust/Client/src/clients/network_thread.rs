use std::net::SocketAddr;
use std::ops::Add;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::guarantees::ReliabilityGuarantees;
use cheetah_common::commands::{BothDirectionCommand, CommandWithReliabilityGuarantees};
use cheetah_common::network::{ConnectionStatus, NetworkChannel};
use cheetah_protocol::frame::member_private_key::MemberPrivateKey;
use cheetah_protocol::frame::ConnectionId;
use cheetah_protocol::{RoomId, RoomMemberId};

use crate::clients::{ClientRequest, SharedClientStatistics};

///
/// Управление сетевым клиентом, связывает поток unity и поток сетевого клиента
///
#[derive(Debug)]
pub struct NetworkChannelManager {
	connection_status: Arc<Mutex<ConnectionStatus>>,
	commands_from_server: Sender<CommandWithReliabilityGuarantees>,
	channel: NetworkChannel,
	request_from_controller: Receiver<ClientRequest>,
	protocol_time_offset_for_test: Option<Duration>,
	shared_statistics: SharedClientStatistics,
	running: bool,
	pub server_time: Arc<Mutex<Option<u64>>>,
}

#[derive(Debug)]
pub struct C2SCommandWithChannel {
	pub channel_type: ReliabilityGuarantees,
	pub command: C2SCommand,
}

impl NetworkChannelManager {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		connection_id: ConnectionId,
		server_address: SocketAddr,
		member_id: RoomMemberId,
		room_id: RoomId,
		private_key: MemberPrivateKey,
		in_commands: Sender<CommandWithReliabilityGuarantees>,
		connection_status: Arc<Mutex<ConnectionStatus>>,
		receiver: Receiver<ClientRequest>,
		shared_statistics: SharedClientStatistics,
		server_time: Arc<Mutex<Option<u64>>>,
	) -> std::io::Result<NetworkChannelManager> {
		Ok(NetworkChannelManager {
			connection_status,
			commands_from_server: in_commands,
			channel: NetworkChannel::new(connection_id, false, private_key, member_id, room_id, server_address, Instant::now())?,
			request_from_controller: receiver,
			protocol_time_offset_for_test: None,
			shared_statistics,
			running: false,
			server_time,
		})
	}

	pub fn run(mut self) {
		self.running = true;
		while self.running {
			let now = self.get_now_time();
			self.channel.cycle(now);
			self.update_server_time();
			self.commands_from_server();
			self.request_from_controller();
			self.update_state();
			thread::sleep(Duration::from_millis(7));
		}
		tracing::info!("Close network_thread client");
	}

	fn update_server_time(&mut self) {
		let mut server_time: MutexGuard<'_, Option<u64>> = self.server_time.lock().unwrap();
		match self.channel.protocol.rtt.remote_time {
			None => {}
			Some(time) => {
				server_time.replace(time);
			}
		}
	}

	///
	/// Текущее время, с учетом коррекции для тестов
	///
	fn get_now_time(&mut self) -> Instant {
		let now = Instant::now();
		if let Some(offset) = self.protocol_time_offset_for_test {
			now.add(offset)
		} else {
			now
		}
	}

	///
	/// Обработка команд с сервера
	///
	fn commands_from_server(&mut self) {
		let in_commands_from_protocol = self.channel.protocol.input_data_handler.get_ready_commands();
		for command in in_commands_from_protocol {
			match self.commands_from_server.send(command.clone()) {
				Ok(_) => {}
				Err(e) => {
					self.running = false;
					tracing::error!("[client] error send command from server {:?}", e);
				}
			}
		}
	}

	///
	/// Обработка команд из контроллера
	///
	fn request_from_controller(&mut self) {
		while let Ok(command) = self.request_from_controller.try_recv() {
			match command {
				ClientRequest::Close(reason) => {
					self.channel.protocol.disconnect_by_command.disconnect(reason);
					let now = Instant::now();
					self.channel.cycle(now);
					self.running = false;
					tracing::info!("[client] ClientRequest::Close");
				}
				ClientRequest::SetProtocolTimeOffsetForTest(duration) => {
					self.protocol_time_offset_for_test = Some(duration);
				}
				ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion) => self.channel.socket_wrapper.config_emulator(|emulator| {
					emulator.configure_rtt(rtt, rtt_dispersion);
				}),
				ClientRequest::ConfigureDropEmulation(drop_probability, drop_time) => self.channel.socket_wrapper.config_emulator(|emulator| {
					emulator.configure_drop(drop_probability, drop_time);
				}),
				ClientRequest::ResetEmulation => {
					self.channel.socket_wrapper.reset_emulator();
				}
				ClientRequest::SendCommandToServer(command) => {
					self.channel.protocol.output_data_producer.add_command(command.channel_type, BothDirectionCommand::C2S(command.command));
				}
			}
		}
	}

	///
	/// Обновление статистики для контроллера
	///
	#[allow(clippy::cast_possible_truncation)]
	fn update_state(&mut self) {
		let protocol = &mut self.channel.protocol;
		self.shared_statistics.current_frame_id.store(protocol.next_frame_id, Ordering::Relaxed);
		self.shared_statistics
			.rtt_in_ms
			.store(protocol.rtt.get_rtt().unwrap_or_else(|| Duration::from_millis(0)).as_millis() as u64, Ordering::Relaxed);
		self.shared_statistics
			.rtt_in_ms
			.store(protocol.rtt.get_rtt().unwrap_or_else(|| Duration::from_millis(0)).as_millis() as u64, Ordering::Relaxed);

		let channel = &self.channel.socket_wrapper;
		self.shared_statistics.recv_packet_count.store(channel.recv_packet_count, Ordering::Relaxed);
		self.shared_statistics.send_packet_count.store(channel.send_packet_count, Ordering::Relaxed);
		self.shared_statistics.send_size.store(channel.send_size, Ordering::Relaxed);
		self.shared_statistics.recv_size.store(channel.recv_size, Ordering::Relaxed);
		*self.connection_status.lock().unwrap() = self.channel.state.clone();
	}
}
