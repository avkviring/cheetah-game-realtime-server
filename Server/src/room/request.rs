use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use mio::net::TcpStream;

use cheetah_relay_common::constants::{ClientId, GlobalObjectId};
use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::room::access::AccessGroups;

use crate::network::server::tcp::room::TcpRoom;
use crate::room::clients::ClientConnectError;
use crate::room::room::Room;

/// Исполнение внешних запросов к комнате
/// Запросы передаются через mpsc:Receiver
pub struct RoomRequests {
	receiver: Receiver<RoomRequest>
}


#[derive(Debug)]
pub enum RoomRequest {
	AddWaitingClient(HashValue, AccessGroups),
	TCPClientConnect(HashValue, TcpStream, Vec<u8>),
	GetClients(Sender<Vec<ClientInfo>>),
	GetObjects(Sender<Vec<GlobalObjectId>>),
}

pub struct ClientInfo {
	pub id: ClientId,
	pub hash: HashValue,
}

impl RoomRequests {
	pub fn new(receiver: Receiver<RoomRequest>) -> RoomRequests {
		RoomRequests {
			receiver,
		}
	}
	
	
	pub fn cycle(&mut self, room: &mut Room, tcp_room: &mut TcpRoom) {
		let command = self.receiver.recv_timeout(Duration::from_millis(1));
		match command {
			Ok(command) => {
				match command {
					RoomRequest::TCPClientConnect(hash, stream, data) => {
						self.do_tcp_client_connect(room, tcp_room, &hash, stream, data.as_slice());
					}
					RoomRequest::AddWaitingClient(client_hash, access_group) => {
						self.do_add_waiting_client(room, &client_hash, access_group);
					}
					RoomRequest::GetClients(sender) => {
						self.do_get_clients(room, sender);
					}
					RoomRequest::GetObjects(sender) => {
						self.do_get_objects(room, sender)
					}
				}
			}
			Err(e) => {
				match e {
					RecvTimeoutError::Timeout => {}
					RecvTimeoutError::Disconnected => {
						log::error!("room requests: Error in receive command: {}",e)
					}
				}
			}
		}
	}
	
	fn do_get_objects(&self, room: &Room, sender: Sender<Vec<GlobalObjectId>>) {
		log::trace!("room requests: get objects from room {}", room.hash);
		let result = sender.send(room.objects.get_object_ids());
		match result {
			Ok(_) => {}
			Err(e) => {
				log::trace!("room requests: get objects - error on send result {} from room {}", e,room.hash);
			}
		}
	}
	
	fn do_get_clients(&self, room: &mut Room, sender: Sender<Vec<ClientInfo>>) {
		log::trace!("room requests: get clients from room {}", room.hash);
		let clients = room.clients.clients.values().map(|c| {
			ClientInfo {
				id: c.configuration.id,
				hash: c.configuration.hash.clone(),
			}
		}).collect();
		let result = sender.send(clients);
		match result {
			Ok(_) => {}
			Err(e) => {
				log::trace!("room requests: get clients - error on send result {} from room {}", e, room.hash);
			}
		}
	}
	
	fn do_add_waiting_client(&self, room: &mut Room, client_hash: &HashValue, access_group: AccessGroups) {
		log::trace!("room requests: add waiting client {} to room {}", client_hash, room.hash);
		room.add_client_to_waiting_list(&client_hash, access_group);
	}
	
	fn do_tcp_client_connect(&self, room: &mut Room, tcp_room: &mut TcpRoom, hash: &HashValue, stream: TcpStream, data: &[u8]) {
		let client = room.client_connect(&hash);
		match client {
			Ok(client) => {
				log::trace!("room requests: connect client {} to room {}", client.configuration.hash, room.hash);
				let result = tcp_room
					.new_connection(room, client.clone(), stream, data);
				if result.is_err() {
					room.client_disconnect(&*client);
				}
			}
			Err(e) => {
				match e {
					ClientConnectError::ClientNotInWaitingList => {
						log::error!("room requests: error connect client {:?} to room {}",e, room.hash)
					}
				}
			}
		}
	}
}