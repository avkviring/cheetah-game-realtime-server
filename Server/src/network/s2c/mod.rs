use std::collections::{HashMap, VecDeque};

use cheetah_relay_common::constants::{ClientId, FieldID};
use cheetah_relay_common::network::command::Encoder;
use cheetah_relay_common::network::command::event::EventCommand;
use cheetah_relay_common::network::command::float_counter::SetFloatCounterCommand;
use cheetah_relay_common::network::command::long_counter::SetLongCounterCommand;
use cheetah_relay_common::network::command::structure::SetStructCommand;
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::network::command::upload::UploadGameObjectS2CCommand;
use cheetah_relay_common::network::niobuffer::NioBuffer;
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
}


#[derive(Debug, Clone, PartialEq)]
pub enum S2CCommandUnion {
    UploadObject(UploadGameObjectS2CCommand),
    Unload(UnloadGameObjectCommand),
    Event(EventCommand),
    UpdateFloat(SetFloatCounterCommand),
    UpdateLong(SetLongCounterCommand),
    UpdateStruct(SetStructCommand),
}


impl S2CCommandUnion {
    fn get_code(&self) -> u8 {
        match self {
            S2CCommandUnion::UploadObject(_) => 1,
            S2CCommandUnion::Unload(_) => 2,
            S2CCommandUnion::UpdateLong(_) => 3,
            S2CCommandUnion::UpdateFloat(_) => 4,
            S2CCommandUnion::Event(_) => 6,
            S2CCommandUnion::UpdateStruct(_) => 5
        }
    }
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
        let affected_clients = AffectedClients::new_from_clients(clients, &game_object.access_groups);
        let command = UploadGameObjectS2CCommand {
            id: game_object.id,
            fields: game_object.fields.clone(),
        };
        self.push(&affected_clients, S2CCommandUnion::UploadObject(command));
    }

    fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
        let affected_clients = AffectedClients::new_from_clients(clients, &game_object.access_groups);
        let command = UnloadGameObjectCommand { global_object_id: game_object.id };
        self.push(&affected_clients, S2CCommandUnion::Unload(command));
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
                let command = UploadGameObjectS2CCommand {
                    id: o.id,
                    fields: o.fields.clone(),
                };
                self.push(&affected_clients, S2CCommandUnion::UploadObject(command))
            })
    }

    fn on_client_disconnect(&mut self, client: &Client) {
        self.commands_by_client.remove(&client.configuration.id);
    }


    fn on_object_long_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
        let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.access_groups);
        let command = SetLongCounterCommand {
            global_object_id: game_object.id,
            field_id,
            value: game_object.get_long_counter(field_id),
        };
        self.push(&affected_clients, S2CCommandUnion::UpdateLong(command))
    }

    fn on_object_float_counter_change(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
        let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.access_groups);
        let command = SetFloatCounterCommand {
            global_object_id: game_object.id,
            field_id,
            value: game_object.get_float_counter(field_id),
        };
        self.push(&affected_clients, S2CCommandUnion::UpdateFloat(command))
    }

    fn on_object_event_fired(&mut self, field_id: FieldID, event_data: &[u8], game_object: &GameObject, clients: &Clients) {
        let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.access_groups);
        let command = EventCommand {
            global_object_id: game_object.id,
            field_id,
            event: Vec::from(event_data),
        };
        self.push(&affected_clients, S2CCommandUnion::Event(command))
    }

    fn on_object_struct_updated(&mut self, field_id: FieldID, game_object: &GameObject, clients: &Clients) {
        let affected_clients = AffectedClients::new_from_clients(&clients, &game_object.access_groups);
        let command = SetStructCommand {
            global_object_id: game_object.id,
            field_id,
            data: game_object.get_struct(field_id).unwrap().clone(),
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
                    S2CCommandUnion::Unload(command) => command.encode(buffer),
                    S2CCommandUnion::Event(command) => command.encode(buffer),
                    S2CCommandUnion::UpdateFloat(command) => command.encode(buffer),
                    S2CCommandUnion::UpdateLong(command) => command.encode(buffer),
                    S2CCommandUnion::UpdateStruct(command) => command.encode(buffer),
                    S2CCommandUnion::UploadObject(command) => command.encode(buffer)
                };
                match result {
                    Ok(_) => {}
                    Err(_) => {
                        commands.push_front(command);
                        break;
                    }
                }
            }
        }
    }
}
