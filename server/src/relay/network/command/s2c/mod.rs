use std::collections::{HashMap, VecDeque};

use crate::main;
use crate::relay::network::command::c2s::trace_c2s_command;
use crate::relay::network::command::s2c::delete_game_object::DeleteGameObjectS2CCommand;
use crate::relay::network::command::s2c::event::EventS2CCommand;
use crate::relay::network::command::s2c::update_float_counter::UpdateFloatCounterS2CCommand;
use crate::relay::network::command::s2c::update_long_counter::UpdateLongCounterS2CCommand;
use crate::relay::network::command::s2c::update_struct::UpdateStructS2CCommand;
use crate::relay::network::command::s2c::upload_object::UploadGameObjectS2CCommand;
use crate::relay::network::types::niobuffer::{NioBuffer, NioBufferError};
use crate::relay::room::clients::{Client, Clients};
use crate::relay::room::groups::AccessGroups;
use crate::relay::room::listener::RoomListener;
use crate::relay::room::objects::object::{FieldID, GameObject};
use crate::relay::room::objects::Objects;
use crate::relay::room::room::{ClientId, Room};

pub mod delete_game_object;
pub mod update_long_counter;
pub mod update_float_counter;
pub mod event;
pub mod update_struct;
pub mod upload_object;


#[derive(Debug, Clone, PartialEq)]
pub enum S2CCommandUnion {
	DeleteObject(DeleteGameObjectS2CCommand),
	Event(EventS2CCommand),
	UpdateFloat(UpdateFloatCounterS2CCommand),
	UpdateLong(UpdateLongCounterS2CCommand),
	UpdateStruct(UpdateStructS2CCommand),
	UploadObject(UploadGameObjectS2CCommand),
}


impl S2CCommandUnion {
	fn get_code(&self) -> u8 {
		match self {
			S2CCommandUnion::UploadObject(_) => 1,
			S2CCommandUnion::DeleteObject(_) => 2,
			S2CCommandUnion::UpdateLong(_) => 3,
			S2CCommandUnion::UpdateFloat(_) => 4,
			S2CCommandUnion::Event(_) => 6,
			S2CCommandUnion::UpdateStruct(_) => 5
		}
	}
}


/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
pub struct S2CCommandCollector {
	pub commands_by_client: HashMap<ClientId, VecDeque<S2CCommandUnion>>,
}

pub trait S2CCommand {
	/// преобразовать команду в поток байт
	/// return true - успешно, false - не достаточно свободного места в buffer
	fn encode(&self, buffer: &mut NioBuffer) -> bool;
}


impl S2CCommandCollector {
	pub fn new() -> Self {
		S2CCommandCollector {
			commands_by_client: Default::default(),
		}
	}
	
	fn push(&mut self, affected_client: &AffectedClients, command: S2CCommandUnion) {
		log::trace!("S2C {:?} : {:?}", command, affected_client);
		affected_client.clients.iter().for_each(|client| {
			let buffer = self.commands_by_client.get_mut(&client);
			match buffer {
				None => {
					log::error!("s2c command collector: client {} not found in commands_by_client", client)
				}
				Some(buffers) => {
					buffers.push_back(command.clone());
				}
			}
		})
	}
}

impl RoomListener for S2CCommandCollector {
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		let cloned_object = game_object.clone();
		let affected_clients = AffectedClients::new_from_clients(clients, &game_object.groups);
		let command = UploadGameObjectS2CCommand { cloned_object };
		self.push(&affected_clients, S2CCommandUnion::UploadObject(command));
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		let affected_clients = AffectedClients::new_from_clients(clients, &game_object.groups);
		let command = DeleteGameObjectS2CCommand { global_object_id: game_object.id };
		self.push(&affected_clients, S2CCommandUnion::DeleteObject(command));
	}
	
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
		self.commands_by_client.insert(client.configuration.id.clone(), Default::default());
		objects
			.get_objects_by_group_in_create_order(&client.configuration.groups)
			.iter()
			.for_each(|o| {
				let o = o.clone();
				let o = &*o;
				let o = o.borrow();
				let affected_clients = AffectedClients::new_from_client(client);
				let command = UploadGameObjectS2CCommand { cloned_object: o.clone() };
				self.push(&affected_clients, S2CCommandUnion::UploadObject(command))
			})
	}
	
	fn on_client_disconnect(&mut self, client: &Client) {
		self.commands_by_client.remove(&client.configuration.id);
	}
	
	
	fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.groups);
		let command = UpdateLongCounterS2CCommand {
			global_object_id: game_object.id,
			field_id,
			value: game_object.get_long_counter(field_id),
		};
		self.push(&affected_clients, S2CCommandUnion::UpdateLong(command))
	}
	
	fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.groups);
		let command = UpdateFloatCounterS2CCommand {
			global_object_id: game_object.id,
			field_id,
			value: game_object.get_float_counter(field_id),
		};
		self.push(&affected_clients, S2CCommandUnion::UpdateFloat(command))
	}
	
	fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &[u8], game_object: &GameObject, clients: &Clients) {
		let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.groups);
		let command = EventS2CCommand {
			global_object_id: game_object.id,
			field_id,
			event: Vec::from(event_data),
		};
		self.push(&affected_clients, S2CCommandUnion::Event(command))
	}
	
	fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
		let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.groups);
		let command = UpdateStructS2CCommand {
			global_object_id: game_object.id,
			field_id,
			struct_data: game_object.get_struct(field_id).unwrap().clone(),
		};
		self.push(&affected_clients, S2CCommandUnion::UpdateStruct(command));
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

pub fn encode_s2c_commands(buffer: &mut NioBuffer, commands: &mut VecDeque<S2CCommandUnion>) {
	loop {
		match commands.pop_front() {
			None => { break; }
			Some(command) => {
				let result_write_code = buffer.write_u8(command.get_code());
				if let Err(_) = result_write_code {
					commands.push_front(command);
					break;
				}
				
				let result = match &command {
					S2CCommandUnion::DeleteObject(command) => command.encode(buffer),
					S2CCommandUnion::Event(command) => command.encode(buffer),
					S2CCommandUnion::UpdateFloat(command) => command.encode(buffer),
					S2CCommandUnion::UpdateLong(command) => command.encode(buffer),
					S2CCommandUnion::UpdateStruct(command) => command.encode(buffer),
					S2CCommandUnion::UploadObject(command) => command.encode(buffer)
				};
				if !result {
					commands.push_front(command);
					break;
				}
			}
		}
	}
}
