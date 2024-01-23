use cheetah_game_realtime_protocol::{RoomId, RoomMemberId};
use thiserror::Error;

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::field::Field;
use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};

use crate::server::room::object::GameObjectError;
use crate::server::room::Room;
use crate::server::room_registry::RoomNotFoundError;

pub mod create;
pub mod created;
pub mod delete;
pub mod double;
pub mod event;
pub mod field;
pub mod items;
pub mod long;
pub mod room;
pub mod structure;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ServerCommandError {
	#[error("{0:?}")]
	Error(String),

	#[error(transparent)]
	GameObjectError(#[from] GameObjectError),

	#[error("Member {member_id} not owner for game object {object_id:?}")]
	MemberNotOwnerGameObject { object_id: GameObjectId, member_id: RoomMemberId },

	#[error("RoomNotFoundError {0}")]
	RoomNotFound(#[from] RoomNotFoundError),

	#[error("Member with id {0}")]
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
		config {template_id:?} in room {room_id:?}"
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

	#[error("ForwardedCommandPermissionDenied: {msg} sender_member_id={sender_member_id} creator_member_id={creator_member_id}")]
	ForwardedCommandPermissionDenied {
		msg: String,
		sender_member_id: RoomMemberId,
		creator_member_id: RoomMemberId,
	},
}

impl ServerCommandError {
	pub fn log_command_execute_error(&self, command: &C2SCommand, room_id: RoomId, room_member_id: RoomMemberId) {
		tracing::error!("Error execute command: {:?} in room {} from client {} : {:?}", command, room_id, room_member_id, self);
	}

	pub fn log_error(&self, room_id: RoomId, room_member_id: RoomMemberId) {
		tracing::error!("Error in room {:?} for client {:?} : {:?}", room_id, room_member_id, self);
	}
}

pub fn execute(command: &C2SCommand, room: &mut Room, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
	match command {
		C2SCommand::CreateGameObject(command) => create::create_object(command, room, member_id),
		C2SCommand::SetLong(command) => long::set(command, room, member_id),
		C2SCommand::SetDouble(command) => double::set(command, room, member_id),
		C2SCommand::SetStructure(command) => structure::set(command, room, member_id),
		C2SCommand::IncrementLongValue(command) => long::increment(command, room, member_id),
		C2SCommand::IncrementDouble(command) => double::increment(command, room, member_id),
		C2SCommand::Event(command) => event::send(command, room, member_id),
		C2SCommand::Delete(command) => delete::delete(command, room, member_id),
		C2SCommand::AttachToRoom => room::attach_to_room(room, member_id),
		C2SCommand::DetachFromRoom => room::detach_from_room(room, member_id),
		C2SCommand::CreatedGameObject(command) => created::created_object(command, room, member_id),
		C2SCommand::TargetEvent(command) => event::send_target(command, room, member_id),
		C2SCommand::DeleteField(command) => field::delete(command, room, member_id),
		C2SCommand::AddItem(command) => items::add(command, room, member_id),
	}
}

#[cfg(test)]
mod tests {
	use cheetah_game_realtime_protocol::RoomMemberId;

	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	pub(crate) fn setup_two_players() -> (Room, GameObjectId, RoomMemberId, RoomMemberId) {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(0b11);
		let mut room = Room::new(0, template);
		let member_1 = room.register_member(MemberCreateParams::stub(access_groups));
		let member_2 = room.register_member(MemberCreateParams::stub(access_groups));
		let object_id = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups, Default::default()).id;
		(room, object_id, member_1, member_2)
	}

	pub(crate) fn setup_one_player() -> (Room, RoomMemberId, AccessGroups) {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::new(0, template);
		let member_id = room.register_member(MemberCreateParams::stub(access_groups));
		(room, member_id, access_groups)
	}
}
