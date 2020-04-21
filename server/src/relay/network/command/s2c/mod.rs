use std::cell::RefCell;
use std::rc::Rc;

use bytebuffer::ByteBuffer;

use crate::relay::network::command::c2s::upload_game_object::UploadGameObjectC2SCommand;
use crate::relay::network::command::c2s::delete_game_object::DeleteGameObjectC2SCommand;
use crate::relay::network::command::s2c::delete_game_object::DeleteObjectS2CCommand;
use crate::relay::network::command::s2c::event::EventS2CCommand;
use crate::relay::network::command::s2c::update_float_counter::UpdateFloatCounterS2CCommand;
use crate::relay::network::command::s2c::update_long_counter::UpdateLongCounterS2CCommand;
use crate::relay::network::command::s2c::update_struct::UpdateStructS2CCommand;
use crate::relay::network::command::s2c::upload_object::UploadObjectS2CCommand;
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::object::{FieldID, GameObject};
use crate::relay::room::room::{ClientId, Room};

pub mod delete_game_object;
pub mod update_long_counter;
pub mod update_float_counter;
pub mod event;
pub mod update_struct;
pub mod upload_object;


/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
pub struct S2CCommandCollector {
	room: Rc<RefCell<Room>>,
	commands: Vec<Box<dyn S2CCommand>>,
}

pub trait S2CCommand {
	/// получить идентификатор команды
	fn get_command_id(&self) -> u8;
	
	/// список затронутых клиентов
	fn get_affected_clients(&self) -> &AffectedClients;
	
	/// преобразовать команду в поток байт
	fn encode(&self, bytes: &mut ByteBuffer);
}

impl RoomListener for S2CCommandCollector {
	fn on_object_created(&mut self, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		let object = game_object.clone();
		self.commands.push(Box::new(
			UploadObjectS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				cloned_object: object,
			}
		))
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		self.commands.push(Box::new(
			DeleteObjectS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				global_object_id: game_object.id,
			}
		))
	}
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		self.commands.push(Box::new(
			UpdateLongCounterS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				value: game_object.get_long_counter(field_id),
			}
		))
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		self.commands.push(Box::new(
			UpdateFloatCounterS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				value: game_object.get_float_counter(field_id),
			}
		))
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		self.commands.push(Box::new(
			EventS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				event: event_data.clone(),
			}
		))
	}
	
	fn on_object_struct_changed(&mut self, field_id: FieldID, game_object: &GameObject) {
		let cloned_room_rc = self.room.clone();
		let room = cloned_room_rc.borrow();
		self.commands.push(Box::new(
			UpdateStructS2CCommand {
				affected_clients: AffectedClients::new(&room.clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				struct_data: game_object.get_struct(field_id).unwrap().clone(),
			}
		))
	}
}


/// список клиентов, затронутые данной командой
pub struct AffectedClients {
	pub clients: Vec<ClientId>
}

impl AffectedClients {
	pub fn new(clients: &Clients, groups: &AccessGroups) -> AffectedClients {
		let mut affected_clients = vec![];
		for client in clients.get_clients() {
			if groups.contains_any(&client.configuration.groups) {
				affected_clients.push(client.configuration.id);
			}
		}
		AffectedClients {
			clients: affected_clients
		}
	}
}


