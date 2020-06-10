use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use mio::{Events, Interest, Poll, Token};
use mio::event::Event;
use mio::net::TcpStream;

use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::network::tcp::connection::TcpConnection;

use crate::network::c2s::decode_end_execute_c2s_commands;
use crate::network::s2c::{encode_s2c_commands, S2CCommandCollector};
use crate::room::clients::Client;
use crate::room::room::Room;

/// Поддержка TCP на уровне комнаты
pub struct TcpRoom {
	poll: Poll,
	events: Events,
	clients: HashMap<Token, ConnectionWithClient>,
	token_generator: usize,
	s2c_collector: Rc<RefCell<S2CCommandCollector>>,
	schedule_for_close: HashSet<Token>,
}

struct ConnectionWithClient {
	connection: TcpConnection,
	client: Rc<Client>,
}

impl TcpRoom {
	pub fn new(room: &mut Room) -> TcpRoom {
		let collector = Rc::new(RefCell::new(S2CCommandCollector::new()));
		room.listener.add_listener(collector.clone());
		TcpRoom {
			poll: Poll::new().unwrap(),
			events: Events::with_capacity(256),
			clients: Default::default(),
			token_generator: Default::default(),
			s2c_collector: collector,
			schedule_for_close: Default::default(),
		}
	}
	
	/// цикл обработки команд
	pub fn cycle(&mut self, room: &mut Room) {
		self
			.poll
			.poll(&mut self.events, Option::Some(Duration::from_millis(1)))
			.unwrap();
		
		self.prepare_commands_to_clients();
		self.process_events(room);
		self.delete_disconnected_clients(room);
	}
	
	
	///
	/// Кодируем команды для клиентов
	///
	fn prepare_commands_to_clients(&mut self) {
		let mut collector = self.s2c_collector.borrow_mut();
		let poll = &mut self.poll;
		self.clients.iter_mut().for_each(|(token, connectionWithClient)| {
			let client = &mut connectionWithClient.client;
			let commands = collector.commands_by_client.get_mut(&client.configuration.id);
			match commands {
				None => {}
				Some(commands) => {
					let connection = &mut connectionWithClient.connection;
					connection.prepare_commands_for_send(
						poll,
						commands,
						|buffer, command| {
							encode_s2c_commands(buffer, command)
						},
					);
				}
			}
		});
	}
	
	fn process_events(&mut self, room: &mut Room) {
		let poll = &mut self.poll;
		for event in &self.events {
			let token = event.token();
			let connection_with_client = self.clients.get_mut(&token);
			match connection_with_client {
				Some(connection_with_client) => {
					let client = &mut connection_with_client.client;
					let connection = &mut connection_with_client.connection;
					match connection.process_event(
						event,
						poll,
						|buffer| {
							decode_end_execute_c2s_commands(buffer, client, room)
						},
					) {
						Ok(_) => {}
						Err(e) => {
							log::error!("tcp room: closing connect {:?} {:?}", token,e);
							self.schedule_for_close.insert(token);
						}
					}
				}
				None => {
					log::error!("tcp room: client by token not found, token {:?}", token);
				}
			}
		}
	}
	
	
	///
	/// регистрируем нового клиента
	///
	pub fn new_connection(&mut self, room: &mut Room, client: Rc<Client>, stream: TcpStream, data: &[u8]) -> Result<(), String> {
		self.token_generator += 1;
		let token = Token(self.token_generator);
		
		let mut buffer_for_read = NioBuffer::new();
		let write_result = buffer_for_read.write_bytes(data);
		match write_result {
			Ok(_) => {
				let mut connection = TcpConnection::new(stream, buffer_for_read, token);
				connection.process_read_buffer(|buffer| {
					decode_end_execute_c2s_commands(buffer, &client, room)
				});
				connection.watch_read(&mut self.poll).unwrap();
				
				self.clients.insert(token.clone(), ConnectionWithClient {
					client,
					connection,
				});
				Result::Ok(())
			}
			Err(e) => {
				Result::Err(format!("{:?}", e))
			}
		}
	}
	
	
	fn delete_disconnected_clients(&mut self, room: &mut Room) {
		for token_for_delete in self.schedule_for_close.iter() {
			match self.clients.remove(token_for_delete) {
				None => {}
				Some(mut connection_with_client) => {
					let client = &mut connection_with_client.client;
					let connection = &mut connection_with_client.connection;
					log::trace!("tcp room: disconnect client {:?}", client);
					room.client_disconnect(&client.clone());
					connection.stop_watch(&mut self.poll);
				}
			}
		}
	}
}