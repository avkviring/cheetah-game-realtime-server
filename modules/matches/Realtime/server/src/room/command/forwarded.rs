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
					creator_member_id: self.user_id,
				});
			}
		} else {
			return Err(ServerCommandError::MemberNotFound(user_id));
		}

		// check that sender and creator are different
		if user_id == self.user_id {
			return Err(ServerCommandError::ForwardedCommandPermissionDenied {
				msg: "ForwardedCommand sender and creator should be different".to_string(),
				sender_member_id: user_id,
				creator_member_id: self.user_id,
			});
		}

		if let Some(member) = room.members.get(&self.user_id) {
			// check that command creator is not a super member
			if member.template.super_member {
				return Err(ServerCommandError::ForwardedCommandPermissionDenied {
					msg: "only non super members commands can be forwarded".to_string(),
					sender_member_id: user_id,
					creator_member_id: self.user_id,
				});
			}
		} else {
			// check that command creator exists in the room
			return Err(ServerCommandError::MemberNotFound(self.user_id));
		}

		// execute forwarded command on behalf of the original member
		execute(&self.c2s, room, self.user_id)
	}
}
