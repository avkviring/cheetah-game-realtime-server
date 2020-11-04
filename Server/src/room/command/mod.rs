use cheetah_relay_common::commands::command::GameObjectCommand;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::udp::protocol::frame::applications::ApplicationCommandChannel;

use crate::room::client::Client;
use crate::room::object::GameObject;
use crate::room::Room;

pub mod event;
pub mod structure;
pub mod create;
pub mod delete;
pub mod long;
pub mod float;


///
/// Выполнение серверной команды
///
pub trait ServerRoomCommandExecutor {
    fn execute(self, room: &mut Room, context: &CommandContext);
}

trait ServerObjectCommandExecutor: GameObjectCommand {
    fn execute(self, object: &mut GameObject, context: &CommandContext);
}


pub struct CommandContext<'a> {
    current_client: Option<&'a Client>,
    channel: ApplicationCommandChannel,
    meta: Option<C2SMetaCommandInformation>,
}


pub fn trace_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
    log::trace!(
        "C2S {:<10} : room {} : client {} : {}",
        command,
        room.hash,
        client.configuration.hash,
        message
    );
}

pub fn error_c2s_command(command: &str, room: &Room, client: &Client, message: String) {
    log::error!(
        "C2S {:<10} : room {} : client {} : {}",
        command,
        room.hash,
        client.configuration.hash,
        message
    );
}