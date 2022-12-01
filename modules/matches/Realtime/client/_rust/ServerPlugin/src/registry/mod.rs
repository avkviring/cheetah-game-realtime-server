use std::collections::HashMap;

use fnv::FnvBuildHasher;

use crate::registry::events::RoomLifecycleEventReader;

pub mod events;
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

    pub fn get_plugin(&mut self, id: ServerPluginId) -> Option<&mut ServerPlugin> {
        return self.plugins.get_mut(&id);
    }
}


pub type ServerPluginId = u16;

pub struct ServerPlugin {
    pub(crate) reader: RoomLifecycleEventReader,
}

impl ServerPlugin {
    pub fn new(server_grpc_addr: String) -> Result<Self, anyhow::Error> {
        Ok(ServerPlugin {
            reader: RoomLifecycleEventReader::from_address(server_grpc_addr)?,
        })
    }
}
