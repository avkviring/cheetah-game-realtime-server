use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use mio::{Events, Interest, Poll, Token};
use mio::net::{TcpListener, TcpStream};

use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::network::niobuffer::NioBuffer;

use crate::room::request::RoomRequest;
use crate::rooms::{Rooms, SendRoomRequestError};

pub mod room;

///
/// Слушает серверный порт и принимает коннекты клиентов
///
/// - принимает новые соединения
/// - читает hash комнаты и клиента
/// - передает соединение в обработчик сети для комнаты
///
pub struct TCPAcceptor {
	running: bool,
	rooms: Arc<Mutex<Rooms>>,
	address: SocketAddr,
	client_token_generator: usize,
	clients: HashMap<Token, IncomingClient>,
	receiver: Receiver<TCPAcceptorRequest>,
}

#[derive(Debug)]
pub enum TCPAcceptorRequest {
	Close
}

const SERVER: Token = Token(1);

impl TCPAcceptor {
	pub fn new(address: String, rooms: Arc<Mutex<Rooms>>, receiver: Receiver<TCPAcceptorRequest>) -> TCPAcceptor {
		TCPAcceptor {
			running: true,
			rooms,
			address: address.parse().unwrap_or_else(|it| panic!("tcp server: cannot parse {} to valid client.network address", it)),
			client_token_generator: 100,
			clients: Default::default(),
			receiver,
		}
	}
	
	pub fn start(&mut self) {
		let mut poll = Poll::new().unwrap_or_else(|_| panic!("tcp server: cannot create client.network pool"));
		let mut events = Events::with_capacity(1024);
		let mut server = TcpListener::bind(self.address).unwrap_or_else(|_| panic!("tcp server: error bind server socket {}", self.address));
		
		
		poll
			.registry()
			.register(
				&mut server,
				SERVER,
				Interest::READABLE | Interest::WRITABLE,
			).unwrap_or_else(|_| panic!("tcp server: error register tcp listener {}", self.address));
		
		while self.running {
			self.process_requests();
			self.process_network_events(&mut poll, &mut events, &mut server)
		}
		
		poll.registry().deregister(&mut server);
	}
	
	fn process_requests(&mut self) {
		match self.receiver.try_recv() {
			Ok(command) => {
				match command {
					TCPAcceptorRequest::Close => { self.running = false; }
				}
			}
			Err(e) => {
				match e {
					TryRecvError::Empty => {}
					TryRecvError::Disconnected => {
						log::error!("tcp server: request disconnected {}", e)
					}
				}
			}
		}
	}
	
	fn process_network_events(&mut self, mut poll: &mut Poll, mut events: &mut Events, server: &mut TcpListener) {
		events.clear();
		poll
			.poll(&mut events, Option::Some(Duration::from_nanos(100)))
			.unwrap_or_else(|e| panic!("tcp server: error poll {}", e));
		
		for event in events.iter() {
			match event.token() {
				SERVER => {
					loop {
						match server.accept() {
							Ok((stream, _)) => {
								log::trace!("tcp server: accept new connection");
								self.register_new_client(&mut poll, stream)
							}
							Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
								break;
							}
							Err(e) => {
								log::error!("tcp server: error accept new client {}", e);
							}
						}
					}
				}
				token => {
					if event.is_readable() {
						self.read_hashes_from_client(&token, &poll);
					}
				}
			}
		}
	}
	
	
	/// TODO - удалять клиентов без хеша (например если за 20 секунд данные не пришли - удаляем)
	fn read_hashes_from_client(&mut self, token: &Token, poll: &Poll) {
		let clients = &mut self.clients;
		let mut client = clients.remove(token).unwrap();
		let stream = &mut client.stream;
		let capacity = client.read_data.len();
		let available_in_buffer = capacity - client.read_count;
		let result = stream.read(&mut client.read_data[client.read_count..capacity]);
		match result {
			Ok(size) => {
				if size == available_in_buffer {
					log::error!("tcp server: overflow input buffer - part data will be lost");
				}
				
				client.read_count += size;
				if TCPAcceptor::is_read_hashes(client.read_count) {
					let rooms = &*self.rooms.clone();
					let room_hash = HashValue::from(&client.read_data[0..HashValue::SIZE]);
					let client_hash = HashValue::from(&client.read_data[HashValue::SIZE..HashValue::SIZE * 2]);
					let mut stream = client.stream;
					poll.registry().deregister(&mut stream);
					let result_send_request = rooms
						.lock()
						.unwrap()
						.send_room_request(
							&room_hash,
							RoomRequest::TCPClientConnect(
								client_hash,
								stream,
								Vec::from(&client.read_data[HashValue::SIZE * 2..client.read_count]),
							),
						);
					match result_send_request {
						Ok(_) => {}
						Err(e) => {
							match e {
								SendRoomRequestError::RoomNotFound => {
									log::error!("tcp server: room not found {}", room_hash);
								}
								SendRoomRequestError::SendError(_) => {
									log::error!("tcp server: send request error");
								}
							}
						}
					}
				} else {
					clients.insert(token.clone(), client);
				}
			}
			Err(e) => {
				log::error!("tcp server: read hash from client error {}",e);
				clients.remove(token);
			}
		}
	}
	
	fn register_new_client(&mut self, poll: &mut Poll, mut stream: TcpStream) {
		let token = Token(self.client_token_generator);
		self.client_token_generator += 1;
		poll
			.registry()
			.register(
				&mut stream,
				token,
				Interest::READABLE,
			).unwrap_or_else(|_| log::error!("Error register client tcp listener"));
		
		self.clients.insert(
			token,
			IncomingClient {
				stream,
				read_data: [0; NioBuffer::NIO_BUFFER_CAPACITY],
				read_count: 0,
			});
	}
	
	/// прочитаны ли hash комнаты и hash клиента
	fn is_read_hashes(count: usize) -> bool {
		count >= HashValue::SIZE * 2
	}
}


struct IncomingClient {
	stream: TcpStream,
	read_data: [u8; NioBuffer::NIO_BUFFER_CAPACITY],
	read_count: usize,
}



