use crate::room::command::ServerCommandError;
use crate::room::Room;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::types::forwarded::ForwardedCommand;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::constants::GameObjectTemplateId;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::RoomMemberId;
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

	pub(crate) fn should_forward(&self, command: &C2SCommand, member_id: RoomMemberId) -> bool {
		// check non super member
		if let Some(member) = self.members.get(&member_id) {
			if member.template.super_member {
				return false;
			}
		}

		if let C2SCommand::Forwarded(_) = command {
			false
		} else {
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

	pub(crate) fn forward_to_super_members(&mut self, command: &C2SCommand, creator_id: RoomMemberId) -> Result<(), ServerCommandError> {
		let s2c = S2CCommandWithMeta {
			field: command.get_field(),
			creator: creator_id,
			command: S2CCommand::Forwarded(Box::new(ForwardedCommand {
				creator: creator_id,
				c2s: command.clone(),
			})),
		};

		self.send_to_members(
			AccessGroups::super_group(),
			self.get_object_template_id(command),
			slice::from_ref(&s2c),
			|member| member.template.super_member,
		)
	}

	fn get_object_template_id(&self, command: &C2SCommand) -> Option<GameObjectTemplateId> {
		if let Some(object_id) = command.get_object_id() {
			self.get_object(object_id).ok().map(|object| object.template_id)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::room::forward::ForwardConfig;
	use crate::room::template::config::{MemberTemplate, RoomTemplate};
	use crate::room::Room;
	use cheetah_common::commands::c2s::C2SCommand;
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::field::SetFieldCommand;
	use cheetah_common::commands::types::forwarded::ForwardedCommand;
	use cheetah_common::commands::{CommandTypeId, FieldValue};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::RoomMemberId;

	#[test]
	fn should_not_forward_from_super_member() {
		let (room, _member, super_member) = setup();
		let command = C2SCommand::AttachToRoom;
		assert!(!room.should_forward(&command, super_member));
	}

	#[test]
	fn should_not_forward_already_forwarded() {
		let (room, member, super_member) = setup();
		let command = C2SCommand::Forwarded(Box::new(ForwardedCommand {
			creator: member,
			c2s: C2SCommand::AttachToRoom,
		}));
		assert!(!room.should_forward(&command, super_member));
	}

	#[test]
	fn should_not_forward_config() {
		let (mut room, member, _super_member) = setup();
		room.put_forwarded_command_config(ForwardConfig {
			command_type_id: CommandTypeId::SetLong,
			field_id: Some(1_u16),
			object_template_id: None,
		});
		let command = C2SCommand::SetField(SetFieldCommand {
			object_id: Default::default(),
			field_id: 2_u16,
			value: FieldValue::Long(1),
		});
		assert!(!room.should_forward(&command, member));
	}

	#[test]
	fn should_forward_config() {
		let (mut room, member, _super_member) = setup();
		room.put_forwarded_command_config(ForwardConfig {
			command_type_id: CommandTypeId::SetLong,
			field_id: Some(1_u16),
			object_template_id: None,
		});
		let command = C2SCommand::SetField(SetFieldCommand {
			object_id: Default::default(),
			field_id: 1_u16,
			value: FieldValue::Long(1),
		});
		assert!(room.should_forward(&command, member));
	}

	#[test]
	fn should_forward_only_to_super_members() {
		let (mut room, member_1, super_member) = setup();
		let command = C2SCommand::AttachToRoom;
		let member_2 = room.register_member(MemberTemplate::stub(AccessGroups(10)));
		room.test_mark_as_connected(super_member).unwrap();
		room.test_mark_as_connected(member_2).unwrap();

		room.forward_to_super_members(&command, member_1).unwrap();

		assert!(room.test_get_member_out_commands(member_2).is_empty());
		assert_eq!(
			S2CCommand::Forwarded(Box::new(ForwardedCommand {
				creator: member_1,
				c2s: command,
			})),
			room.test_get_member_out_commands(super_member)[0]
		);
	}

	fn setup() -> (Room, RoomMemberId, RoomMemberId) {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		let super_member_1 = room.register_member(MemberTemplate::new_super_member());
		(room, member_1, super_member_1)
	}
}
