use thiserror::Error;

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::constants::GameObjectTemplateId;
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::room::object::{Field, GameObjectError};
use crate::room::Room;

pub mod create;
pub mod created;
pub mod delete;
pub mod double;
pub mod event;
pub mod field;
pub mod long;
pub mod room;
pub mod structure;

///
/// Выполнение серверной команды
///
pub trait ServerCommandExecutor {
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError>;
}

#[derive(Error, Debug)]
pub enum ServerCommandError {
	#[error("{:?}",.0)]
	Error(String),

	#[error("{error:?}")]
	GameObjectError {
		#[from]
		error: GameObjectError,
	},

	#[error("Member {member_id:?} not owner for game object {object_id:?}")]
	MemberNotOwnerGameObject {
		object_id: GameObjectId,
		member_id: RoomMemberId,
	},

	#[error("Member with id {:?}",.0)]
	MemberNotFound(RoomMemberId),

	#[error(
		"Member {member_id:?} with group {member_access_group:?} cannot access to \
	object {object_id:?} with group {object_access_group:?} in room {room_id:?}"
	)]
	MemberCannotAccessToObject {
		room_id: RoomId,
		member_id: RoomMemberId,
		object_id: GameObjectId,
		member_access_group: AccessGroups,
		object_access_group: AccessGroups,
	},

	#[error(
		"Member {member_id:?} cannot access to field {field:?} in object {object_id:?} with \
		template {template_id:?} in room {room_id:?}"
	)]
	MemberCannotAccessToObjectField {
		room_id: RoomId,
		member_id: RoomMemberId,
		object_id: GameObjectId,
		template_id: GameObjectTemplateId,
		field: Field,
	},

	#[error("Game object with id {object_id:?} ")]
	GameObjectNotFound { object_id: GameObjectId },
}

impl ServerCommandError {
	pub fn log_command_execute_error(&self, command: &C2SCommand, room_id: RoomId, room_member_id: RoomMemberId) {
		tracing::error!(
			"Error execute command: {:?} in room {} from client {} : {:?}",
			command,
			room_id,
			room_member_id,
			self
		);
	}

	pub fn log_error(&self, room_id: RoomId, room_member_id: RoomMemberId) {
		tracing::error!("Error in room {:?} for client {:?} : {:?}", room_id, room_member_id, self);
	}
}

pub fn execute(command: &C2SCommand, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
	match command {
		C2SCommand::Create(command) => command.execute(room, user_id),
		C2SCommand::SetLong(command) => command.execute(room, user_id),
		C2SCommand::IncrementLongValue(command) => command.execute(room, user_id),
		C2SCommand::CompareAndSetLong(command) => command.execute(room, user_id),
		C2SCommand::SetDouble(command) => command.execute(room, user_id),
		C2SCommand::IncrementDouble(command) => command.execute(room, user_id),
		C2SCommand::SetStructure(command) => command.execute(room, user_id),
		C2SCommand::Event(command) => command.execute(room, user_id),
		C2SCommand::Delete(command) => command.execute(room, user_id),
		C2SCommand::AttachToRoom => room::attach_to_room(room, user_id),
		C2SCommand::DetachFromRoom => room::detach_from_room(room, user_id),
		C2SCommand::Created(command) => command.execute(room, user_id),
		C2SCommand::TargetEvent(command) => command.execute(room, user_id),
		C2SCommand::DeleteField(command) => command.execute(room, user_id),
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;

	pub fn setup_two_players() -> (Room, GameObjectId, RoomMemberId, RoomMemberId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b11);
		let mut room = Room::from_template(template);
		let user_1 = room.register_member(MemberTemplate::stub(access_groups));
		let user_2 = room.register_member(MemberTemplate::stub(access_groups));
		let object_id = room.test_create_object(user_1, access_groups).id.clone();
		(room, object_id, user_1, user_2)
	}

	pub fn setup_one_player() -> (Room, RoomMemberId, AccessGroups) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let user_id = room.register_member(MemberTemplate::stub(access_groups));
		(room, user_id, access_groups)
	}
}
