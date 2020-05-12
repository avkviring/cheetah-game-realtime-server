use std::collections::HashMap;
use std::io;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use mio::{Events, Interest, Poll, Token};
use mio::net::{TcpListener, TcpStream};

use crate::relay::network::types::hash::HashValue;
use crate::relay::room::request::RoomRequest;
use crate::relay::rooms::{Rooms, SendRoomRequestError};
use crate::relay::network::types::niobuffer::NioBuffer;

pub mod room_tcp;

pub struct TCPServer {
	rooms: Arc<Mutex<Rooms>>,
	addr: SocketAddr,
	client_token_generator: usize,
	clients: HashMap<Token, IncomingClient>,
}

const SERVER: Token = Token(1);

impl TCPServer {
	pub fn new(addr: String, rooms: Arc<Mutex<Rooms>>) -> TCPServer {
		TCPServer {
			rooms,
			addr: addr.parse().unwrap_or_else(|it| panic!("tcp server: cannot parse {} to valid network address", it)),
			client_token_generator: 100,
			clients: Default::default(),
		}
	}
	
	pub fn start(&mut self) {
		let mut poll = Poll::new().unwrap_or_else(|_| panic!("tcp server: cannot create network pool"));
		let mut events = Events::with_capacity(1024);
		let mut server = TcpListener::bind(self.addr.clone()).unwrap_or_else(|_| panic!("tcp server: error bind server socket {}", self.addr));
		
		
		poll
			.registry()
			.register(
				&mut server,
				SERVER,
				Interest::READABLE | Interest::WRITABLE,
			).unwrap_or_else(|_| panic!("tcp server: error register tcp listener {}", self.addr));
		
		
		loop {
			events.clear();
			poll
				.poll(&mut events, None)
				.unwrap_or_else(|e| panic!("tcp server: error poll {}", e));
			
			for event in events.iter() {
				match event.token() {
					SERVER => {
						loop {
							match server.accept() {
								Ok((stream, addr)) => {
									log::trace!("tcp server: accept new connection");
									self.register_new_client(&mut poll, stream, addr)
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
	}
	
	
	/// TODO - удалять клиентов без хеша (например если за 20 секунд данные не пришли - удаляем)
	fn read_hashes_from_client(&mut self, token: &Token, poll: &Poll) {
		let clients = &mut self.clients;
		let mut client = clients.remove(token).unwrap();
		let stream = &mut client.connection.stream;
		let capacity = client.read_data.len();
		let available_in_buffer = capacity - client.read_count;
		let result = stream.read(&mut client.read_data[client.read_count..capacity]);
		match result {
			Ok(size) => {
				if size == available_in_buffer {
					log::error!("tcp server: overflow input buffer - part data will be lost");
				}
				
				client.read_count += size;
				if TCPServer::is_read_hashes(client.read_count) {
					let rooms = &*self.rooms.clone();
					let room_hash = HashValue::from(&client.read_data[0..HashValue::SIZE]);
					let client_hash = HashValue::from(&client.read_data[HashValue::SIZE..HashValue::SIZE * 2]);
					let mut stream = client.connection.stream;
					poll.registry().deregister(&mut stream);
					let result_send_request = rooms
						.lock()
						.unwrap()
						.send_room_request(
							&room_hash,
							RoomRequest::TCPClientConnect(
								client_hash,
								stream,
								client.connection.addr.clone(),
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
	
	fn register_new_client(&mut self, poll: &mut Poll, stream: TcpStream, addr: SocketAddr,
	) {
		let mut connection = ClientConnection {
			stream,
			addr,
		};
		let token = Token(self.client_token_generator);
		self.client_token_generator += 1;
		poll
			.registry()
			.register(
				&mut connection.stream,
				token,
				Interest::READABLE,
			).unwrap_or_else(|_| log::error!("Error register client tcp listener"));
		
		self.clients.insert(
			token,
			IncomingClient {
				connection,
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
	connection: ClientConnection,
	read_data: [u8; NioBuffer::NIO_BUFFER_CAPACITY],
	read_count: usize,
}


struct ClientConnection {
	stream: TcpStream,
	addr: SocketAddr,
}

