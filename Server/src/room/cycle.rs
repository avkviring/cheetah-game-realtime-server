use std::sync::mpsc::Receiver;

use crate::network::server::tcp::room::TcpRoom;
use crate::room::request::{RoomRequest, RoomRequests};
use crate::room::room::Room;
use cheetah_relay_common::network::hash::HashValue;

/// Основной цикл обработки команд игровой комнаты
/// Структура является единственным владельцем комнаты
pub struct RoomCycle {
	requests: RoomRequests,
	tcp: TcpRoom,
	room: Room,
}

impl RoomCycle {
	pub fn new(room_hash: HashValue, receiver: Receiver<RoomRequest>) -> RoomCycle {
		let mut room = Room::new(room_hash);
		let tcp = TcpRoom::new(&mut room);
		RoomCycle {
			requests: RoomRequests::new(receiver),
			room,
			tcp,
		}
	}
	
	pub fn start(&mut self) {
		loop {
			self.tcp.cycle(&mut self.room);
			self.requests.cycle(&mut self.room, &mut self.tcp);
		}
	}
}