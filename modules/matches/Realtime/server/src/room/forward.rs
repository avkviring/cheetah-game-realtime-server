use crate::room::command::ServerCommandError;
use crate::room::object::S2CCommandWithFieldInfo;
use crate::room::Room;
use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::RoomMemberId;
use std::slice;

impl Room {
	pub(crate) fn is_forwarded(&self, _command: &C2SCommand) -> bool {
		// todo
		false
	}

	pub(crate) fn forward_to_super_members(&mut self, command: &C2SCommand, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let s2c = S2CCommandWithFieldInfo {
			field: command.get_field(),
			command: S2CCommand::Forwarded(Box::new(ForwardedCommand {
				user_id,
				c2s: command.clone(),
			})),
		};

		let mut object_template: Option<GameObjectTemplateId> = None;
		if let Some(object_id) = command.get_object_id() {
			object_template = self.get_object(&object_id).ok().map(|object| object.template_id);
		}

		self.send_to_members(AccessGroups::super_group(), object_template, slice::from_ref(&s2c), |_user| true)
	}
}
