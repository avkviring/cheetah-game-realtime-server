use std::sync::mpsc::{Receiver, Sender, TryRecvError};

use mio::net::TcpStream;

use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::commands::hash::HashValue;
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::clients::ClientConnectError;
use crate::room::Room;
use crate::room::objects::id::ServerGameObjectId;

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
	GetObjects(Sender<Vec<ServerGameObjectId>>),
	Destroy,
}

#[derive(Debug)]
pub struct ClientInfo {
	pub id: ClientId,
	pub hash: HashValue,
}

#[derive(Debug)]
pub enum RequestResult {
	///
	/// Удалить комнату
	///
	Destroy,
	///
	/// Нет запросов
	///
	EmptyRequest,
	
	///
	/// Обработан запрос
	///
	SingleRequest,
}

impl RoomRequests {
	pub fn new(receiver: Receiver<RoomRequest>) -> RoomRequests {
		RoomRequests {
			receiver,
		}
	}
	
	
	// pub fn cycle(&mut self, room: &mut Room, tcp_room: &mut TcpRoom) -> Result<RequestResult, TryRecvError> {
	// 	let command = self.receiver.try_recv();
	// 	match command {
	// 		Ok(command) => {
	// 			match command {
	// 				RoomRequest::TCPClientConnect(hash, stream, data) => {
	// 					self.do_tcp_client_connect(room, tcp_room, &hash, stream, data.as_slice());
	// 					Result::Ok(RequestResult::SingleRequest)
	// 				}
	// 				RoomRequest::AddWaitingClient(client_hash, access_group) => {
	// 					self.do_add_waiting_client(room, &client_hash, access_group);
	// 					Result::Ok(RequestResult::SingleRequest)
	// 				}
	// 				RoomRequest::GetClients(sender) => {
	// 					self.do_get_clients(room, sender);
	// 					Result::Ok(RequestResult::SingleRequest)
	// 				}
	// 				RoomRequest::GetObjects(sender) => {
	// 					self.do_get_objects(room, sender);
	// 					Result::Ok(RequestResult::SingleRequest)
	// 				}
	// 				RoomRequest::Destroy => {
	// 					Result::Ok(RequestResult::Destroy)
	// 				}
	// 			}
	// 		}
	// 		Err(e) => {
	// 			match e {
	// 				TryRecvError::Empty => {
	// 					Result::Ok(RequestResult::EmptyRequest)
	// 				}
	// 				TryRecvError::Disconnected => {
	// 					Result::Err(e)
	// 				}
	// 			}
	// 		}
	// 	}
	// }
	
	fn do_get_objects(&self, room: &Room, sender: Sender<Vec<ServerGameObjectId>>) {
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
}