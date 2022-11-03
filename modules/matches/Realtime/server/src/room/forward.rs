use crate::room::command::ServerCommandError;
use crate::room::object::S2CCommandWithFieldInfo;
use crate::room::Room;
use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
use cheetah_matches_realtime_common::commands::CommandTypeId;
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::RoomMemberId;
use std::slice;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ForwardConfig {
	pub(crate) command_type_id: CommandTypeId,
	pub(crate) field_id: Option<FieldId>,
	pub(crate) object_template_id: Option<GameObjectTemplateId>,
}

impl Room {
	pub(crate) fn put_forwarded_command_config(&mut self, config: ForwardConfig) {
		self.forward_configs.insert(config);
	}

	pub(crate) fn should_forward(&self, command: &C2SCommand, user_id: RoomMemberId) -> bool {
		// check non super member
		if let Some(member) = self.members.get(&user_id) {
			if member.template.super_member {
				return false;
			}
		}

		match command {
			C2SCommand::Forwarded(_) => false,
			_ => {
				let mut config = ForwardConfig {
					command_type_id: command.get_type_id(),
					field_id: command.get_field_id(),
					object_template_id: self.get_object_template_id(command),
				};
				if self.forward_configs.contains(&config) {
					return true;
				}

				config.object_template_id = None;
				if self.forward_configs.contains(&config) {
					return true;
				}

				config.field_id = None;
				self.forward_configs.contains(&config)
			}
		}
	}

	pub(crate) fn forward_to_super_members(&mut self, command: &C2SCommand, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let s2c = S2CCommandWithFieldInfo {
			field: command.get_field(),
			command: S2CCommand::Forwarded(Box::new(ForwardedCommand {
				user_id,
				c2s: command.clone(),
			})),
		};

		self.send_to_members(
			AccessGroups::super_group(),
			self.get_object_template_id(command),
			slice::from_ref(&s2c),
			|_user| true,
		)
	}

	fn get_object_template_id(&self, command: &C2SCommand) -> Option<GameObjectTemplateId> {
		if let Some(object_id) = command.get_object_id() {
			self.get_object(&object_id).ok().map(|object| object.template_id)
		} else {
			None
		}
	}
}
