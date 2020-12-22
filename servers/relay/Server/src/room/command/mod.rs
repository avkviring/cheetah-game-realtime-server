use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::Room;

pub mod create;
pub mod created;
pub mod delete;
pub mod event;
pub mod float;
pub mod long;
pub mod room;
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
	match command {
		C2SCommand::Create(command) => command.execute(room, user_public_key),
		C2SCommand::SetLong(command) => command.execute(room, user_public_key),
		C2SCommand::IncrementLongValue(command) => command.execute(room, user_public_key),
		C2SCommand::CompareAndSetLongValue(command) => command.execute(room, user_public_key),
		C2SCommand::SetFloat(command) => command.execute(room, user_public_key),
		C2SCommand::IncrementFloatCounter(command) => command.execute(room, user_public_key),
		C2SCommand::SetStruct(command) => command.execute(room, user_public_key),
		C2SCommand::Event(command) => command.execute(room, user_public_key),
		C2SCommand::Delete(command) => command.execute(room, user_public_key),
		C2SCommand::AttachToRoom => room::attach_to_room(room, user_public_key),
		C2SCommand::Created(command) => command.execute(room, user_public_key),
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::template::config::RoomTemplate;
	use crate::room::Room;

	pub fn setup() -> (Room, GameObjectId, UserPublicKey, UserPublicKey) {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);
		let user_1 = template.configure_user(1, access_groups);
		let user_2 = template.configure_user(2, access_groups);
		let mut room = Room::from_template(template);
		let object_id = room.create_object(&user_1, access_groups).id.clone();
		(room, object_id, user_1, user_2)
	}
}
