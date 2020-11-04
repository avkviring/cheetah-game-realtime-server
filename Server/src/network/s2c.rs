use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::command::S2CCommandWithMeta;
use cheetah_relay_common::constants::ClientId;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel};
use cheetah_relay_common::udp::protocol::relay::RelayProtocol;

use crate::room::client::{Client, Clients};
use crate::room::command::CommandContext;
use crate::room::object::GameObject;

/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
#[derive(Debug)]
pub struct S2CCommandCollector {
    pub clients: Rc<RefCell<Clients>>,
    pub commands_by_client: HashMap<ClientId, RelayProtocol>,
}


impl S2CCommandCollector {
    pub fn new(clients: Rc<RefCell<Clients>>) -> Self {
        Self {
            clients,
            commands_by_client: Default::default(),
        }
    }


    pub fn collect<F: FnMut(&ClientId) -> S2CCommandUnion>(&mut self, object: &GameObject, context: &CommandContext, mut command_factory: F) {
        // let clients = self.clients.clone();
        // let clients = clients.as_ref().borrow();
        // let affected_clients = AffectedClients::new_from_clients(current_client, &clients, access_groups);
        // let meta_from_client = &meta.unwrap();
        // affected_clients.clients.iter().for_each(|client| {
        //     let relay_protocol = self.commands_by_client.get_mut(&client);
        //     match relay_protocol {
        //         None => log::error!(
        //             "s2c command collector: client {} not found in commands_by_client",
        //             client
        //         ),
        //         Some(protocol) => {
        //             let command = command_factory(client);
        //             log::trace!("S2C {:?} : {:?}", command, access_groups);
        //             let meta = S2CMetaCommandInformation::new(*client, meta_from_client);
        //             protocol.out_commands_collector.add_command(channel.clone(), ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta { meta, command }));
        //         }
        //     }
        // })
    }
}

// impl RoomListener for S2CCommandCollector {
//     fn set_current_client(&mut self, client: Rc<Client>) {
//         self.current_client = Option::Some(client);
//     }
//
//     fn unset_current_client(&mut self) {
//         self.current_client = Option::None
//     }
//
//     fn set_current_meta_info(&mut self, meta: Rc<C2SMetaCommandInformation>) {
//         self.current_meta_info = Option::Some(meta.clone())
//     }
//
//     fn unset_current_meta_info(&mut self) {
//         self.current_meta_info = Option::None
//     }
//
//     fn on_object_created(&mut self, game_object: &GameObject, clients: &Clients) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, clients, &game_object.access_groups), |client|
//             S2CCommandUnion::Load(
//                 LoadGameObjectCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     template: game_object.template,
//                     access_groups: game_object.access_groups.clone(),
//                     fields: game_object.fields.clone(),
//                 }),
//         );
//     }
//
//     fn on_object_delete(&mut self, game_object: &GameObject, clients: &Clients) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, clients, &game_object.access_groups), |client|
//             {
//                 S2CCommandUnion::Unload(
//                     UnloadGameObjectCommand {
//                         object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     })
//             },
//         );
//     }
//
//     fn on_client_connect(&mut self, client: &Client, objects: &Objects) {
//         self.commands_by_client
//             .insert(client.configuration.id, Default::default());
//         objects
//             .get_objects_by_group_in_create_order(&client.configuration.groups)
//             .iter()
//             .for_each(|o| {
//                 let o = o.clone();
//                 let o = &*o;
//                 let o = o.borrow();
//                 let affected_clients = AffectedClients::new_from_client(client);
//                 self.collect(affected_clients, |client|
//                     {
//                         S2CCommandUnion::Load(
//                             LoadGameObjectCommand {
//                                 object_id: o.id.to_client_object_id(Option::Some(*client)),
//                                 template: o.template,
//                                 access_groups: o.access_groups.clone(),
//                                 fields: o.fields.clone(),
//                             })
//                     },
//                 )
//             })
//     }
//
//     fn on_client_disconnect(&mut self, client: &Client) {
//         self.commands_by_client.remove(&client.configuration.id);
//     }
//
//     fn on_object_long_counter_change(
//         &mut self,
//         field_id: FieldID,
//         game_object: &GameObject,
//         clients: &Clients,
//     ) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::SetLongCounter(
//                 SetLongCounterCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     value: game_object.get_long_counter(field_id),
//                 }),
//         )
//     }
//
//     fn on_object_float_counter_change(
//         &mut self,
//         field_id: FieldID,
//         game_object: &GameObject,
//         clients: &Clients,
//     ) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::SetFloatCounter(
//                 SetFloat64CounterCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     value: game_object.get_float_counter(field_id),
//                 }),
//         )
//     }
//
//     fn on_object_event_fired(
//         &mut self,
//         field_id: FieldID,
//         event_data: &[u8],
//         game_object: &GameObject,
//         clients: &Clients,
//     ) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::Event(
//                 EventCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     event: Vec::from(event_data),
//                 }),
//         )
//     }
//
//     fn on_object_struct_updated(
//         &mut self,
//         field_id: FieldID,
//         game_object: &GameObject,
//         clients: &Clients,
//     ) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::SetStruct(
//                 StructureCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     structure: game_object.get_struct(field_id).unwrap().clone(),
//                 }),
//         )
//     }
//
//     fn on_object_long_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::SetLongCounter(
//                 SetLongCounterCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     value: game_object.get_long_counter(field_id),
//                 }),
//         )
//     }
//
//     fn on_object_float_counter_set(&mut self, field_id: u16, game_object: &GameObject, clients: &Clients) {
//         self.collect(AffectedClients::new_from_clients(&self.current_client, &clients, &game_object.access_groups), |client|
//             S2CCommandUnion::SetFloatCounter(
//                 SetFloat64CounterCommand {
//                     object_id: game_object.id.to_client_object_id(Option::Some(*client)),
//                     field_id,
//                     value: game_object.get_float_counter(field_id),
//                 }),
//         )
//     }
// }

/// список клиентов, затронутые данной командой
#[derive(Debug, PartialEq)]
pub struct AffectedClients {
    pub clients: Vec<ClientId>,
}

impl AffectedClients {
    pub fn new_from_clients(current_client: Option<&Client>, clients: &Clients, groups: &AccessGroups) -> AffectedClients {
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