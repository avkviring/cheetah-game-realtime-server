use std::io::{Error, Read, Write};
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime};

use mio::{Events, Interest, Poll, Token};
use mio::net::TcpStream;

use cheetah_relay_common::network::command::{CommandCode, Decoder, Encoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::network::command::long_counter::{IncrementLongCounterC2SCommand, SetLongCounterCommand};
use cheetah_relay_common::network::command::structure::SetStructCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::{UploadGameObjectC2SCommand, UploadGameObjectS2CCommand};
use cheetah_relay_common::network::niobuffer::{NioBuffer, NioBufferError};
use cheetah_relay_common::network::tcp::connection::{TcpConnection, TcpConnectionError};

use crate::client::{Client, NetworkStatus};
use crate::client::command::{C2SCommandUnion, decode_command, encode_command, S2CCommandUnion};
use crate::client::command::C2SCommandUnion::{Event, SetStruct};

const token: Token = Token(0);

#[derive(Debug)]
pub struct TCPClient {
	pub server_address: String,
	pub connection: Option<TcpConnection>,
	pub poll: Poll,
	pub events: Events,
	pub connection_start_time: Instant,
	status: Status,
}

#[derive(Debug)]
enum Status {
	None,
	Connecting,
	Connected,
	Disconnected,
}

impl TCPClient {
	pub fn new(server_address: String) -> TCPClient {
		let poll = Poll::new().unwrap();
		let events = Events::with_capacity(1024);
		
		TCPClient {
			server_address,
			connection: Option::None,
			poll,
			events,
			connection_start_time: Instant::now(),
			status: Status::None,
		}
	}
	pub fn cycle(&mut self, client: &mut Client) -> NetworkStatus {
		self.status = match self.status {
			Status::None => {
				self.create_connect(client)
			}
			Status::Connecting => {
				self.wait_connect(client)
			}
			Status::Connected => {
				match self.process_network_events(client) {
					Ok(_) => { Status::Connected }
					Err(_) => { Status::Disconnected }
				}
			}
			Status::Disconnected => {
				Status::Disconnected
			}
		};
		
		match self.status {
			Status::None => { NetworkStatus::Connecting }
			Status::Connecting => { NetworkStatus::Connecting }
			Status::Connected => { NetworkStatus::OnLine }
			Status::Disconnected => { NetworkStatus::Disconnected }
		}
	}
	
	fn wait_connect(&mut self, client: &mut Client) -> Status {
		match self.process_network_events(client) {
			Ok(count_event) => {
				if count_event > 0 {
					Status::Connected
				} else if self.connection_start_time.elapsed() > Duration::from_secs(3) {
					Status::Disconnected
				} else {
					Status::Connecting
				}
			}
			Err(_) => {
				Status::Disconnected
			}
		}
	}
	
	
	fn process_network_events(&mut self, client: &mut Client) -> Result<usize, ()> {
		self.prepare_to_send_commands(client)?;
		let poll = &mut self.poll;
		let connection = self.connection.as_mut().unwrap();
		match poll.poll(&mut self.events, Option::Some(Duration::from_millis(1))) {
			Ok(_) => {
				let mut count_success_event = 0;
				for event in &self.events {
					match connection.process_event(
						event,
						poll,
						|buffer| { decode_command(buffer, &mut client.commands_from_server) }) {
						Ok(_) => {}
						Err(_) => {
							return Result::Err(());
						}
					}
					count_success_event += 1;
				}
				Result::Ok(count_success_event)
			}
			Err(e) => {
				log::error!("tcp_client process_events pool {:?}", e);
				Result::Err(())
			}
		}
	}
	
	
	///
	/// Преобразовать существующие команды в поток байт
	/// и подписаться на Write событие записи сокета если есть данные для записи
	///
	fn prepare_to_send_commands(&mut self, client: &mut Client) -> Result<(), ()> {
		let poll = &mut self.poll;
		let connection = self.connection.as_mut().unwrap();
		match connection.prepare_commands_for_send(
			poll,
			&mut client.scheduled_command_to_server,
			|buffer, command| {
				encode_command(buffer, command)
			})
		{
			Ok(_) => {
				Result::Ok(())
			}
			Err(_) => {
				Result::Err(())
			}
		}
	}
	
	
	fn create_connect(&mut self, client: &mut Client) -> Status {
		let address = SocketAddr::from_str(self.server_address.as_str()).unwrap();
		match TcpStream::connect(address) {
			Ok(stream) => {
				let mut connection = TcpConnection::new(stream, NioBuffer::new(), token);
				connection.write_buffer.clear();
				connection.write_buffer.write_bytes(&client.room_hash.value).unwrap();
				connection.write_buffer.write_bytes(&client.client_hash.value).unwrap();
				connection.write_buffer.flip();
				connection.watch_write_and_read(&mut self.poll).unwrap();
				self.connection = Option::Some(connection);
				self.connection_start_time = Instant::now();
				Status::Connecting
			}
			Err(e) => {
				log::error!("tcp client connect {:?}", e);
				Status::Disconnected
			}
		}
	}
}