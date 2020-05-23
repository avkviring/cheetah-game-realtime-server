use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use mio::{Events, Interest, Poll, Token};
use mio::net::TcpStream;

use crate::network::command::c2s::decode_end_execute_c2s_commands;
use crate::network::command::s2c::{encode_s2c_commands, S2CCommand, S2CCommandCollector, S2CCommandUnion};
use crate::network::types::niobuffer::{NioBuffer, NioBufferError};
use crate::room::clients::Client;
use crate::room::room::Room;

/// Поддержка TCP на уровне комнаты
pub struct TCPRoom {
	poll: Poll,
	events: Events,
	clients: HashMap<Token, TcpClient>,
	token_generator: usize,
	tmp_buffer: Box<[u8; NioBuffer::NIO_BUFFER_CAPACITY]>,
	s2c_collector: Rc<RefCell<S2CCommandCollector>>,
	schedule_for_close: HashSet<Token>,
}

struct TcpClient {
	client: Rc<Client>,
	stream: TcpStream,
	addr: SocketAddr,
	readed: Box<NioBuffer>,
	buffer_for_write: Box<NioBuffer>,
}


impl TCPRoom {
	pub fn new(room: &mut Room) -> TCPRoom {
		let collector = Rc::new(RefCell::new(S2CCommandCollector::new()));
		room.listener.add_listener(collector.clone());
		TCPRoom {
			poll: Poll::new().unwrap(),
			events: Events::with_capacity(256),
			clients: Default::default(),
			token_generator: Default::default(),
			tmp_buffer: Box::new([0; NioBuffer::NIO_BUFFER_CAPACITY]),
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
		self.process_events(room);
		self.delete_disconnected_clients(room);
	}
	
	fn process_events(&mut self, room: &mut Room) {
		for event in &self.events {
			let token = event.token();
			let tcp_client = self.clients.get_mut(&token);
			match tcp_client {
				Some(tcp_client) => {
					if event.is_readable() {
						let mut stream = &tcp_client.stream;
						let read_result = stream.read(&mut *self.tmp_buffer);
						match read_result {
							Ok(0) => {
								self.schedule_for_close.insert(token.clone());
							}
							Ok(size) => {
								let buffer = &mut tcp_client.readed;
								if size > buffer.remaining() {
									log::error!("tcp room: packet is too big, packet size = {}, remaining in buffer {}", size, buffer.remaining());
									buffer.clear();
								} else {
									buffer.write_bytes(&self.tmp_buffer[0..size]).unwrap();
									buffer.flip();
									decode_end_execute_c2s_commands(buffer, &tcp_client.client, room);
									buffer.compact();
								}
							}
							Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
								// пропускаем
							}
							Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
								// пропускаем
							}
							Err(e) => {
								log::info!("tcp room: error read from client - closing client {:?}", e);
								self.schedule_for_close.insert(token.clone());
							}
						}
					}
					if event.is_writable() {
						let buffer_for_write = &mut tcp_client.buffer_for_write;
						let mut collector = self.s2c_collector.borrow_mut();
						let commands = collector.commands_by_client.get_mut(&tcp_client.client.configuration.id);
						if let Some(commands) = commands { encode_s2c_commands(buffer_for_write, commands) }
						buffer_for_write.flip();
						if buffer_for_write.has_remaining() {
							let result = tcp_client.stream.write(buffer_for_write.to_slice());
							match result {
								Ok(result) => {
									let position_result = buffer_for_write.set_position(buffer_for_write.position() + result);
									if let Err(_) = position_result {
										log::error!("tcp room: write buffer - error when set new position {:?}", token);
									}
								}
								Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
									// пропускаем
								}
								Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
									// пропускаем
								}
								Err(e) => {
									log::error!("tcp room: write buffer error - closing client {:?}", e);
									self.schedule_for_close.insert(token.clone());
								}
							}
							buffer_for_write.compact();
						}
					}
				}
				None => {
					log::error!("tcp room: client by token not found, token {:?}", token);
				}
			}
		}
	}
	
	fn delete_disconnected_clients(&mut self, room: &mut Room) {
		for token_for_delete in self.schedule_for_close.iter() {
			let client = self.clients.remove(token_for_delete);
			match client {
				None => {}
				Some(mut client) => {
					log::trace!("tcp room: disconnect client {}", client.client.configuration.hash);
					room.client_disconnect(&client.client.clone());
					self.poll.registry().deregister(&mut client.stream).unwrap();
				}
			}
		}
	}
	
	/// регистрируем нового клиента
	pub fn add_client(&mut self, room: &mut Room, client: Rc<Client>, mut stream: TcpStream, addr: SocketAddr, data: &[u8]) -> Result<(), String> {
		self.token_generator += 1;
		let next_token = Token(self.token_generator);
		let result = self.poll.registry().register(
			&mut stream,
			next_token.clone(),
			Interest::READABLE | Interest::WRITABLE,
		);
		
		match result {
			Ok(_) => {
				let mut buffer = NioBuffer::new();
				let write_result = buffer.write_bytes(data);
				match write_result {
					Ok(_) => {
						buffer.flip();
						decode_end_execute_c2s_commands(&mut buffer, &client, room);
						buffer.compact();
						let tcp_client = TcpClient {
							client,
							stream,
							addr,
							readed: Box::new(buffer),
							buffer_for_write: Box::new(NioBuffer::new()),
						};
						self.clients.insert(next_token.clone(), tcp_client);
						Result::Ok(())
					}
					Err(e) => {
						Result::Err(format!("{:?}", e))
					}
				}
			}
			Err(e) => {
				Result::Err(format!("{:?}", e))
			}
		}
	}
}