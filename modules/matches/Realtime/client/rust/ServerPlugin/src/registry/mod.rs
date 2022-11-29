use std::collections::HashMap;

use fnv::FnvBuildHasher;

use crate::registry::created_room::CreateRoomEventReader;

pub mod created_room;
#[cfg(test)]
pub mod stubs;

pub type RoomId = u64;

#[derive(Default)]
pub struct Registry {
    plugins: HashMap<ServerPluginId, ServerPlugin, FnvBuildHasher>,
    server_plugin_generator_id: ServerPluginId,
}

impl Registry {

    pub fn register_plugin(&mut self, server_plugin: ServerPlugin) -> ServerPluginId {
        let plugin_id = self.server_plugin_generator_id;
        self.plugins.insert(plugin_id, server_plugin);
        self.server_plugin_generator_id += 1;
        plugin_id
    }
}


pub type ServerPluginId = u16;

pub struct ServerPlugin {
    create_room_event_reader: CreateRoomEventReader,
}

impl ServerPlugin {
    pub fn new(server_grpc_addr: String) -> Result<Self, anyhow::Error> {
        Ok(ServerPlugin {
            create_room_event_reader: CreateRoomEventReader::from_address(server_grpc_addr)?,
        })
    }
}
