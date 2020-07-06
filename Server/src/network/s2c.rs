use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use cheetah_relay_common::constants::{ClientId, FieldID};
use cheetah_relay_common::network::command::{CommandCode, Encoder};
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::SetFloat64CounterCommand;
use cheetah_relay_common::network::command::long_counter::SetLongCounterCommand;
use cheetah_relay_common::network::command::structure::StructureCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::network::niobuffer::{NioBuffer, NioBufferError};
use cheetah_relay_common::room::access::AccessGroups;

use crate::room::clients::{Client, Clients};
use crate::room::listener::RoomListener;
use crate::room::objects::object::GameObject;
use crate::room::objects::Objects;

/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
pub struct S2CCommandCollector {
	pub commands_by_client: HashMap<ClientId, VecDeque<S2CCommandUnion>>,
	pub current_client: Option<Rc<Client>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum S2CCommandUnion {
	UploadGameObject(UploadGameObjectCommand),
	UnloadGameObject(UnloadGameObjectCommand),
	Event(EventCommand),
	SetFloatCounter(SetFloat64CounterCommand),
	SetLongCounter(SetLongCounterCommand),
	Struct(StructureCommand),
}

impl S2CCommandUnion {
	fn get_code(&self) -> u8 {
		match self {
			S2CCommandUnion::UploadGameObject(_) => UploadGameObjectCommand::COMMAND_CODE,
			S2CCommandUnion::UnloadGameObject(_) => UnloadGameObjectCommand::COMMAND_CODE,
			S2CCommandUnion::SetLongCounter(_) => SetLongCounterCommand::COMMAND_CODE,
			S2CCommandUnion::SetFloatCounter(_) => SetFloat64CounterCommand::COMMAND_CODE,
			S2CCommandUnion::Event(_) => EventCommand::COMMAND_CODE,
			S2CCommandUnion::Struct(_) => StructureCommand::COMMAND_CODE,
		}
	}
}


impl Default for S2CCommandCollector {
	fn default() -> Self {
		S2CCommandCollector {
			commands_by_client: Default::default(),
			current_client: Default::default(),
		}
	}
}

impl S2CCommandCollector {
	fn push<F: FnMut(&ClientId) -> S2CCommandUnion>(&mut self, affected_client: AffectedClients, mut command_factory: F) {
		affected_client.clients.iter().for_each(|client| {
			let buffer = self.commands_by_client.get_mut(&client);
			match buffer {
				None => log::error!(
                    "s2c command collector: client {} not found in commands_by_client",
                    client
                ),
				Some(buffers) => {
					let command = command_factory(client);
					log::trace!("S2C {:?} : {:?}", command, affected_client);
					buffers.push_back(command);
				}
			}
		})
	}
}

impl RoomListener for S2CCommandCollector {
	fn set_current_client(&mut self, client: Rc<Client>) {
		self.current_client = Option::Some(client);
	}
	
	fn unset_current_client(&mut self) {
		self.current_client = Option::None
	}
	
	fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
		self.push(AffectedClients::new_from_clients(&self.current_client, clients, &game_object.access_groups), |client|
			S2CCommandUnion::UploadGameObject(
				UploadGameObjectCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					access_groups: game_object.access_groups.clone(),
					fields: game_object.fields.clone(),
				}),
		);
	}
	
	fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
		self.push(AffectedClients::new_from_clients(&self.current_client, clients, &game_object.access_groups), |client|
			{
				S2CCommandUnion::UnloadGameObject(
					UnloadGameObjectCommand {
						object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					})
			},
		);
	}
	
	fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
		self.commands_by_client
			.insert(client.configuration.id.clone(), Default::default());
		objects
			.get_objects_by_group_in_create_order(&client.configuration.groups)
			.iter()
			.for_each(|o| {
				let o = o.clone();
				let o = &*o;
				let o = o.borrow();
				let affected_clients = AffectedClients::new_from_client(client);
				self.push(affected_clients, |client|
					{
						S2CCommandUnion::UploadGameObject(
							UploadGameObjectCommand {
								object_id: o.id.to_client_object_id(Option::Some(*client)),
								access_groups: o.access_groups.clone(),
								fields: o.fields.clone(),
							})
					},
				)
			})
	}
	
	fn on_client_disconnect(&mut self, client: &Client) {
		self.commands_by_client.remove(&client.configuration.id);
	}
	
	fn on_object_long_counter_change(
		&mut self,
		field_id: FieldID,
		game_object: &GameObject,
		clients: &Clients,
	) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::SetLongCounter(
				SetLongCounterCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					value: game_object.get_long_counter(field_id),
				}),
		)
	}
	
	fn on_object_float_counter_change(
		&mut self,
		field_id: FieldID,
		game_object: &GameObject,
		clients: &Clients,
	) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::SetFloatCounter(
				SetFloat64CounterCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					value: game_object.get_float_counter(field_id),
				}),
		)
	}
	
	fn on_object_event_fired(
		&mut self,
		field_id: FieldID,
		event_data: &[u8],
		game_object: &GameObject,
		clients: &Clients,
	) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::Event(
				EventCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					event: Vec::from(event_data),
				}),
		)
	}
	
	fn on_object_struct_updated(
		&mut self,
		field_id: FieldID,
		game_object: &GameObject,
		clients: &Clients,
	) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::Struct(
				StructureCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					structure: game_object.get_struct(field_id).unwrap().clone(),
				}),
		)
	}
	
	fn on_object_long_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::SetLongCounter(
				SetLongCounterCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					value: game_object.get_long_counter(field_id),
				}),
		)
	}
	
	fn on_object_float_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
		self.push(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
			S2CCommandUnion::SetFloatCounter(
				SetFloat64CounterCommand {
					object_id: game_object.id.to_client_object_id(Option::Some(*client)),
					field_id,
					value: game_object.get_float_counter(field_id),
				}),
		)
	}
}

/// список клиентов, затронутые данной командой
#[derive(Debug, PartialEq)]
pub struct AffectedClients {
	pub clients: Vec<ClientId>,
}

impl AffectedClients {
	pub fn new_from_clients(current_client: &Option<Rc<Client>>, clients: &Clients, groups: &AccessGroups) -> AffectedClients {
		let mut affected_clients = vec![];
		
		let current_client_id = match current_client {
			None => { 0 }
			Some(client) => { client.configuration.id }
		};
		
		for client in clients.get_clients() {
			if current_client_id == client.configuration.id {
				continue;
			}
			if groups.contains_any(&client.configuration.groups) {
				affected_clients.push(client.configuration.id);
			}
		}
		AffectedClients {
			clients: affected_clients,
		}
	}
	
	pub fn new_from_client(client: &Client) -> AffectedClients {
		AffectedClients {
			clients: vec![client.configuration.id],
		}
	}
}

pub fn encode_s2c_commands(buffer: &mut NioBuffer, command: &S2CCommandUnion) -> Result<(), NioBufferError> {
	buffer.write_u8(command.get_code())?;
	match &command {
		S2CCommandUnion::UnloadGameObject(command) => command.encode(buffer),
		S2CCommandUnion::Event(command) => command.encode(buffer),
		S2CCommandUnion::SetFloatCounter(command) => command.encode(buffer),
		S2CCommandUnion::SetLongCounter(command) => command.encode(buffer),
		S2CCommandUnion::Struct(command) => command.encode(buffer),
		S2CCommandUnion::UploadGameObject(command) => command.encode(buffer),
	}
}
