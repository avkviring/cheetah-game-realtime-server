use cheetah_relay_common::commands::command::{C2SCommandUnion, GameObjectCommand};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::room::UserPublicKey;
use cheetah_relay_common::protocol::frame::applications::ApplicationCommandChannel;

use crate::room::{Room, User};
use crate::room::object::GameObject;

pub mod event;
pub mod structure;
pub mod create;
pub mod delete;
pub mod long;
pub mod float;
pub mod load_room;


///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(self, room: &mut dyn Room, user_public_key: &UserPublicKey);
}

pub fn trace_c2s_command(command: &str, room: &dyn Room, user_public_key: &UserPublicKey, message: String) {
	log::trace!(
		"C2S {:<10} : room {} : client {} : {}",
		command,
		room.get_id(),
		user_public_key,
		message
	);
}

pub fn error_c2s_command(command: &str, room: &dyn Room, user_public_key: &UserPublicKey, message: String) {
	log::error!(
		"C2S {:<10} : room {} : client {} : {}",
		command,
		room.get_id(),
		user_public_key,
		message
	);
}

pub fn execute(command: C2SCommandUnion, room: &mut Room, user_public_key: &UserPublicKey) {
	match command {
		C2SCommandUnion::Create(command) => {
			command.execute(room, user_public_key);
		}
		C2SCommandUnion::SetLongCounter(command) => {
			command.execute(room, user_public_key)
		}
		
		C2SCommandUnion::IncrementLongCounter(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::SetFloatCounter(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::IncrementFloatCounter(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::Structure(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::Event(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::Delete(command) => {
			command.execute(room, user_public_key)
		}
		C2SCommandUnion::Test(_) => {}
		C2SCommandUnion::LoadRoom => {
			load_room::load_room(room, user_public_key);
		}
	}
}
