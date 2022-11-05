use crate::room::command::{execute, ServerCommandError, ServerCommandExecutor};
use crate::room::Room;
use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
use cheetah_matches_realtime_common::room::RoomMemberId;

impl ServerCommandExecutor for ForwardedCommand {
	/// execute forwarded command on behalf of the original user
	fn execute(&self, room: &mut Room, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		// check that the command is from a super member
		if let Some(member) = room.members.get(&user_id) {
			if !member.template.super_member {
				return Err(ServerCommandError::ForwardedCommandPermissionDenied {
					msg: "only super members are allowed to send ForwardedCommand".to_string(),
					sender_member_id: user_id,
					creator_member_id: self.creator,
				});
			}
		} else {
			return Err(ServerCommandError::MemberNotFound(user_id));
		}

		// check that sender and creator are different
		if user_id == self.creator {
			return Err(ServerCommandError::ForwardedCommandPermissionDenied {
				msg: "ForwardedCommand sender and creator should be different".to_string(),
				sender_member_id: user_id,
				creator_member_id: self.creator,
			});
		}

		if let Some(member) = room.members.get(&self.creator) {
			// check that command creator is not a super member
			if member.template.super_member {
				return Err(ServerCommandError::ForwardedCommandPermissionDenied {
					msg: "only non super members commands can be forwarded".to_string(),
					sender_member_id: user_id,
					creator_member_id: self.creator,
				});
			}
		} else {
			// check that command creator exists in the room
			return Err(ServerCommandError::MemberNotFound(self.creator));
		}

		// execute forwarded command on behalf of the creator
		execute(&self.c2s, room, self.creator)
	}
}

#[cfg(test)]
mod tests {
	use crate::room::command::ServerCommandError::{ForwardedCommandPermissionDenied, MemberNotFound};
	use crate::room::command::ServerCommandExecutor;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;
	use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
	use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::RoomMemberId;

	#[test]
	fn should_not_execute_when_sender_not_super_member() {
		let (mut room, member_1, _super_member_1, _super_member_2) = setup();
		let command = ForwardedCommand {
			creator: 0,
			c2s: C2SCommand::AttachToRoom,
		};
		assert_eq!(
			ForwardedCommandPermissionDenied {
				msg: "only super members are allowed to send ForwardedCommand".to_string(),
				sender_member_id: member_1,
				creator_member_id: 0,
			},
			command.execute(&mut room, member_1).unwrap_err()
		);
	}

	#[test]
	fn should_not_execute_when_creator_is_super_member() {
		let (mut room, _member_1, super_member_1, super_member_2) = setup();
		let command = ForwardedCommand {
			creator: super_member_2,
			c2s: C2SCommand::AttachToRoom,
		};
		assert_eq!(
			ForwardedCommandPermissionDenied {
				msg: "only non super members commands can be forwarded".to_string(),
				sender_member_id: super_member_1,
				creator_member_id: super_member_2,
			},
			command.execute(&mut room, super_member_1).unwrap_err()
		);
	}

	#[test]
	fn should_not_execute_for_same_sender_and_creator() {
		let (mut room, _member_1, super_member_1, _super_member_2) = setup();
		let command = ForwardedCommand {
			creator: super_member_1,
			c2s: C2SCommand::AttachToRoom,
		};
		assert_eq!(
			ForwardedCommandPermissionDenied {
				msg: "ForwardedCommand sender and creator should be different".to_string(),
				sender_member_id: super_member_1,
				creator_member_id: super_member_1,
			},
			command.execute(&mut room, super_member_1).unwrap_err()
		);
	}

	#[test]
	fn should_not_execute_when_sender_disconnected() {
		let (mut room, member_1, super_member_1, _super_member_2) = setup();
		let command = ForwardedCommand {
			creator: member_1,
			c2s: C2SCommand::AttachToRoom,
		};
		room.disconnect_user(super_member_1).unwrap();
		assert_eq!(MemberNotFound(super_member_1), command.execute(&mut room, super_member_1).unwrap_err());
	}

	#[test]
	fn should_not_execute_when_creator_disconnected() {
		let (mut room, member_1, super_member_1, _super_member_2) = setup();
		let command = ForwardedCommand {
			creator: member_1,
			c2s: C2SCommand::AttachToRoom,
		};
		room.disconnect_user(member_1).unwrap();
		assert_eq!(MemberNotFound(member_1), command.execute(&mut room, super_member_1).unwrap_err());
	}

	#[test]
	fn should_execute() {
		let (mut room, member_1, super_member_1, _super_member_2) = setup();
		let command = ForwardedCommand {
			creator: member_1,
			c2s: C2SCommand::DetachFromRoom,
		};
		if let Err(e) = command.execute(&mut room, super_member_1) {
			panic!("{:?}", e)
		}
		assert!(!room.members.get(&member_1).unwrap().attached);
	}

	fn setup() -> (Room, RoomMemberId, RoomMemberId, RoomMemberId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		let super_member_1 = room.register_member(MemberTemplate::new_super_member());
		let super_member_2 = room.register_member(MemberTemplate::new_super_member());
		(room, member_1, super_member_1, super_member_2)
	}
}
