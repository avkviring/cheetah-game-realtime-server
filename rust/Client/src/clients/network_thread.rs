use std::net::SocketAddr;
use std::ops::Add;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::network::client::{ConnectionStatus, NetworkClient};
use cheetah_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_common::protocol::frame::channel::ChannelType;
use cheetah_common::room::{MemberPrivateKey, RoomId, RoomMemberId};

use crate::clients::{ClientRequest, SharedClientStatistics};

///
/// Управление сетевым клиентом, связывает поток unity и поток сетевого клиента
///
#[derive(Debug)]
pub struct NetworkThreadClient {
	connection_status: Arc<Mutex<ConnectionStatus>>,
	commands_from_server: Sender<CommandWithChannel>,
	udp_client: NetworkClient,
	request_from_controller: Receiver<ClientRequest>,
	protocol_time_offset_for_test: Option<Duration>,
	shared_statistics: SharedClientStatistics,
	running: bool,
	pub server_time: Arc<Mutex<Option<u64>>>,
}

#[derive(Debug)]
pub struct C2SCommandWithChannel {
	pub channel_type: ChannelType,
	pub command: C2SCommand,
}

impl NetworkThreadClient {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		server_address: SocketAddr,
		member_id: RoomMemberId,
		room_id: RoomId,
		private_key: MemberPrivateKey,
		in_commands: Sender<CommandWithChannel>,
		connection_status: Arc<Mutex<ConnectionStatus>>,
		receiver: Receiver<ClientRequest>,
		start_frame_id: u64,
		shared_statistics: SharedClientStatistics,
		server_time: Arc<Mutex<Option<u64>>>,
	) -> std::io::Result<NetworkThreadClient> {
		Ok(NetworkThreadClient {
			connection_status,
			commands_from_server: in_commands,
			udp_client: NetworkClient::new(false, private_key, member_id, room_id, server_address, start_frame_id, Instant::now())?,
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
			self.udp_client.cycle(now);
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
		match self.udp_client.protocol.rtt.remote_time {
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
		let in_commands_from_protocol = self.udp_client.protocol.in_commands_collector.get_ready_commands();
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
					self.udp_client.protocol.disconnect_by_command.disconnect(reason);
					let now = Instant::now();
					self.udp_client.cycle(now);
					self.running = false;
					tracing::info!("[client] ClientRequest::Close");
				}
				ClientRequest::SetProtocolTimeOffsetForTest(duration) => {
					self.protocol_time_offset_for_test = Some(duration);
				}
				ClientRequest::ConfigureRttEmulation(rtt, rtt_dispersion) => self.udp_client.channel.config_emulator(|emulator| {
					emulator.configure_rtt(rtt, rtt_dispersion);
				}),
				ClientRequest::ConfigureDropEmulation(drop_probability, drop_time) => self.udp_client.channel.config_emulator(|emulator| {
					emulator.configure_drop(drop_probability, drop_time);
				}),
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
	#[allow(clippy::cast_possible_truncation)]
	fn update_state(&mut self) {
		let protocol = &mut self.udp_client.protocol;
		self.shared_statistics.current_frame_id.store(protocol.next_frame_id, Ordering::Relaxed);
		self.shared_statistics.rtt_in_ms.store(
			protocol.rtt.get_rtt().unwrap_or_else(|| Duration::from_millis(0)).as_millis() as u64,
			Ordering::Relaxed,
		);
		self.shared_statistics.average_retransmit_frames.store(
			protocol
				.retransmitter
				.statistics
				.get_average_retransmit_frames(Instant::now())
				.unwrap_or(0) as u32,
			Ordering::Relaxed,
		);
		self.shared_statistics.rtt_in_ms.store(
			protocol.rtt.get_rtt().unwrap_or_else(|| Duration::from_millis(0)).as_millis() as u64,
			Ordering::Relaxed,
		);

		let channel = &self.udp_client.channel;
		self.shared_statistics
			.recv_packet_count
			.store(channel.recv_packet_count, Ordering::Relaxed);
		self.shared_statistics
			.send_packet_count
			.store(channel.send_packet_count, Ordering::Relaxed);
		self.shared_statistics.send_size.store(channel.send_size, Ordering::Relaxed);
		self.shared_statistics.recv_size.store(channel.recv_size, Ordering::Relaxed);
		*self.connection_status.lock().unwrap() = self.udp_client.state.clone();
	}
}
