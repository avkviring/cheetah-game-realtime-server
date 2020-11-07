use cheetah_relay_common::commands::command::{C2SCommandUnion, GameObjectCommand};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::protocol::frame::applications::ApplicationCommandChannel;

use crate::room::{Room, User};
use crate::room::object::GameObject;

pub mod event;
pub mod structure;
pub mod create;
pub mod delete;
pub mod long;
pub mod float;


///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(self, room: &mut Room, context: &CommandContext);
}


pub struct CommandContext<'a> {
	pub current_client: Option<&'a User>,
	pub channel: ApplicationCommandChannel,
	pub meta: Option<C2SMetaCommandInformation>,
}


pub fn trace_c2s_command(command: &str, room: &Room, client: &User, message: String) {
	log::trace!(
		"C2S {:<10} : room {} : client {} : {}",
		command,
		room.id,
		client.public_key,
		message
	);
}

pub fn error_c2s_command(command: &str, room: &Room, client: &User, message: String) {
	log::error!(
		"C2S {:<10} : room {} : client {} : {}",
		command,
		room.id,
		client.public_key,
		message
	);
}

pub fn execute(command: C2SCommandUnion, room: &mut Room, context: &CommandContext) {
	match command {
		C2SCommandUnion::Create(command) => {
			command.execute(room, context);
		}
		C2SCommandUnion::SetLongCounter(command) => {
			command.execute(room, context)
		}
		
		C2SCommandUnion::IncrementLongCounter(command) => {
			command.execute(room, context)
		}
		C2SCommandUnion::SetFloatCounter(command) => {
			command.execute(room, context)
		}
		C2SCommandUnion::IncrementFloatCounter(command) => {
			command.execute(room, context)
		}
		C2SCommandUnion::Structure(command) => {
			command.execute(room, context)
		}
		C2SCommandUnion::Event(command) => {
			command.execute(room, context)
		}
		C2SCommandUnion::Delete(command) => {
			command.execute(room, context)
		}
		
		C2SCommandUnion::Test(_) => {}
	}
}
