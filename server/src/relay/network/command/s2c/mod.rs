use std::cell::RefCell;
use std::rc::Rc;

use bytebuffer::ByteBuffer;
use traitcast::TraitcastFrom;

use crate::relay::network::command::s2c::delete_game_object::DeleteGameObjectS2CCommand;
use crate::relay::network::command::s2c::event::EventS2CCommand;
use crate::relay::network::command::s2c::update_float_counter::UpdateFloatCounterS2CCommand;
use crate::relay::network::command::s2c::update_long_counter::UpdateLongCounterS2CCommand;
use crate::relay::network::command::s2c::update_struct::UpdateStructS2CCommand;
use crate::relay::network::command::s2c::upload_object::UploadGameObjectS2CCommand;
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::object::{FieldID, GameObject};
use crate::relay::room::objects::Objects;
use crate::relay::room::room::ClientId;

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
	commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>,
}

pub trait S2CCommand: TraitcastFrom {
	/// получить идентификатор команды
	fn get_command_id(&self) -> u8;
	
	/// список затронутых клиентов
	fn get_affected_clients(&self) -> &AffectedClients;
	
	/// преобразовать команду в поток байт
	fn encode(&self, bytes: &mut ByteBuffer);
}

impl S2CCommandCollector {
	pub fn new(commands: Rc<RefCell<Vec<Box<dyn S2CCommand>>>>) -> Self {
		S2CCommandCollector {
			commands,
		}
	}
	
	fn push(&mut self, command: Box<dyn S2CCommand>) {
		let commands = self.commands.clone();
		let mut commands = (*commands).borrow_mut();
		commands.push(command)
	}
}

impl RoomListener for S2CCommandCollector {
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		let object = game_object.clone();
		self.push(Box::new(
			UploadGameObjectS2CCommand {
				affected_clients: AffectedClients::new_from_clients(clients, &game_object.groups),
				cloned_object: object,
			}
		))
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		self.push(Box::new(
			DeleteGameObjectS2CCommand {
				affected_clients: AffectedClients::new_from_clients(clients, &game_object.groups),
				global_object_id: game_object.id,
			}
		))
	}
	
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
		objects
			.get_objects_by_group_in_create_order(&client.configuration.groups)
			.iter()
			.for_each(|o| {
				let o = o.clone();
				let o = &*o;
				let o = o.borrow();
				self.push(Box::new(
					UploadGameObjectS2CCommand {
						affected_clients: AffectedClients::new_from_client(client),
						cloned_object: o.clone(),
					}
				))
			})
	}
	
	
	fn on_client_disconnect(&mut self, _client: &Client) {
		// ничего не делаем на данный момент
		// так как объекты созданные пользователям
		// удалит room
	}
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		self.push(Box::new(
			UpdateLongCounterS2CCommand {
				affected_clients: AffectedClients::new_from_clients(&clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				value: game_object.get_long_counter(field_id),
			}
		))
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		self.push(Box::new(
			UpdateFloatCounterS2CCommand {
				affected_clients: AffectedClients::new_from_clients(&clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				value: game_object.get_float_counter(field_id),
			}
		))
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &Vec<u8>, game_object: &GameObject, clients: &Clients) {
		self.push(Box::new(
			EventS2CCommand {
				affected_clients: AffectedClients::new_from_clients(&clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				event: event_data.clone(),
			}
		))
	}
	
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		self.push(Box::new(
			UpdateStructS2CCommand {
				affected_clients: AffectedClients::new_from_clients(clients, &game_object.groups),
				global_object_id: game_object.id,
				field_id,
				struct_data: game_object.get_struct(field_id).unwrap().clone(),
			}
		))
	}
}


/// список клиентов, затронутые данной командой
#[derive(Debug, PartialEq)]
pub struct AffectedClients {
	pub clients: Vec<ClientId>
}

impl AffectedClients {
	pub fn new_from_clients(clients: &Clients, groups: &AccessGroups) -> AffectedClients {
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
	
	pub fn new_from_client(client: &Client) -> AffectedClients {
		AffectedClients {
			clients: vec![client.configuration.id]
		}
	}
}


