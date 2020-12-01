use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::Room;

pub mod create;
pub mod delete;
pub mod event;
pub mod float;
pub mod load_room;
pub mod long;
pub mod structure;

///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(self, room: &mut Room, user_public_key: &UserPublicKey);
}

pub fn trace_c2s_command(command: &str, room: &Room, user_public_key: &UserPublicKey, message: String) {
	log::trace!("C2S {:<10} : room {} : client {} : {}", command, room.id, user_public_key, message);
}

pub fn error_c2s_command(command: &str, room: &Room, user_public_key: &UserPublicKey, message: String) {
	log::error!("C2S {:<10} : room {} : client {} : {}", command, room.id, user_public_key, message);
}

pub fn execute(command: C2SCommand, room: &mut Room, user_public_key: &UserPublicKey) {
	log::info!("[room({:?})] u({:?}) -> s {:?}", room.id, user_public_key, command);
	match command {
		C2SCommand::Create(command) => {
			command.execute(room, user_public_key);
		}
		C2SCommand::SetLongValue(command) => command.execute(room, user_public_key),

		C2SCommand::IncrementLongValue(command) => command.execute(room, user_public_key),
		C2SCommand::SetFloatValue(command) => command.execute(room, user_public_key),
		C2SCommand::IncrementFloatCounter(command) => command.execute(room, user_public_key),
		C2SCommand::Structure(command) => command.execute(room, user_public_key),
		C2SCommand::Event(command) => command.execute(room, user_public_key),
		C2SCommand::Delete(command) => command.execute(room, user_public_key),
		C2SCommand::Test(_) => {}
		C2SCommand::AttachToRoom => {
			load_room::attach_to_room(room, user_public_key);
		}
	}
}
