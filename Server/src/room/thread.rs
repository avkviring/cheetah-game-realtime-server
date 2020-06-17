use std::sync::mpsc::Receiver;

use cheetah_relay_common::network::hash::HashValue;

use crate::network::server::tcp::room::TcpRoom;
use crate::room::request::{RequestResult, RoomRequest, RoomRequests};
use crate::room::Room;

/// Основной цикл обработки команд игровой комнаты
/// Структура является единственным владельцем комнаты
pub struct RoomThread {
	requests: RoomRequests,
	tcp: TcpRoom,
	room: Room,
}

impl RoomThread {
	pub fn new(room_hash: HashValue, auto_create_client: bool, receiver: Receiver<RoomRequest>) -> RoomThread {
		let mut room = Room::new(room_hash, auto_create_client);
		let tcp = TcpRoom::new(&mut room);
		RoomThread {
			requests: RoomRequests::new(receiver),
			room,
			tcp,
		}
	}
	
	pub fn run(&mut self) {
		loop {
			self.tcp.cycle(&mut self.room);
			match self.requests.cycle(&mut self.room, &mut self.tcp) {
				Ok(result) => {
					match result {
						RequestResult::Destroy => {
							break;
						}
						RequestResult::EmptyRequest => {}
						RequestResult::SingleRequest => {}
					}
				}
				Err(e) => {
					log::error!("request.cycle send error {:?}",e);
				}
			}
		}
	}
}