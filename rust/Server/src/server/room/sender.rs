use cheetah_game_realtime_protocol::RoomMemberId;

use cheetah_common::commands::guarantees::{ChannelGroup, ReliabilityGuarantees};
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::{BothDirectionCommand, CommandWithChannelType};
use cheetah_common::room::access::AccessGroups;

use crate::server::room::command::ServerCommandError;
use crate::server::room::member::{RoomMember, RoomMemberStatus};
use crate::server::room::Room;

///
/// Методы для отправки команд пользователям
///
impl Room {
	pub fn send_to_members<T>(&mut self, access_groups: AccessGroups, commands: &[S2CCommand], filter: T) -> Result<(), ServerCommandError>
	where
		T: Fn(&RoomMember) -> bool,
	{
		#[cfg(test)]
		for command in commands.iter() {
			self.test_out_commands.push_front((access_groups, command.clone()));
		}

		let channel_type = self.current_channel.as_ref().unwrap_or(&ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let members_for_send = self
			.members
			.values_mut()
			.filter(|member| member.status == RoomMemberStatus::Attached)
			.filter(|member| member.template.groups.contains_any(&access_groups))
			.filter(|member| filter(member));

		for member in members_for_send {
			commands.iter().for_each(|command| {
				member.out_commands.push(CommandWithChannelType {
					channel_type: *channel_type,
					command: BothDirectionCommand::S2C(command.clone()),
				});
			});
		}

		Ok(())
	}

	pub fn send_to_member(&mut self, member_id: &RoomMemberId, commands: &[S2CCommand]) -> Result<(), ServerCommandError> {
		let channel = self.current_channel.unwrap_or(ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let member = self.get_member_mut(member_id)?;

		if member.status == RoomMemberStatus::Attached {
			for command in commands {
				let member = self.get_member_mut(member_id)?;
				member.out_commands.push(CommandWithChannelType {
					channel_type: channel,
					command: BothDirectionCommand::S2C(command.clone()),
				});
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::long::LongField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::field::{Field, FieldType};
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::config::member::MemberCreateParams;
	use crate::server::room::config::room::RoomCreateParams;
	use crate::server::room::Room;

	///
	/// Не посылаем обратную команду, тому кто ее вызвал
	///
	#[test]
	fn should_dont_sent_command_back() {
		let access_groups = AccessGroups(55);
		let field = Field { id: 10, field_type: FieldType::Long };

		let mut room = Room::new(0, RoomCreateParams::default());
		let member_id = room.register_member(MemberCreateParams::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups, Default::default());
		object.access_groups = access_groups;
		object.created = true;
		let object_id = object.id;
		room.mark_as_attached_in_test(member_id).unwrap();

		room.send_command_from_action(object_id, member_id, None, |_| {
			Ok(Some(S2CCommand::SetLong(LongField {
				object_id,
				field_id: field.id,
				value: 0,
			})))
		})
		.unwrap();

		assert!(room.get_member_out_commands_for_test(member_id).is_empty());
	}

	///
	/// Действие не должно выполнится если пользователь не входит в группу объекта
	///
	#[test]
	fn should_skip_action_if_sender_not_in_group() {
		let template = RoomCreateParams::default();
		let access_groups_a = AccessGroups(0b01);
		let access_groups_b = AccessGroups(0b10);
		let mut room = Room::new(0, template);
		let member_1 = room.register_member(MemberCreateParams::stub(access_groups_a));
		let member_2 = room.register_member(MemberCreateParams::stub(access_groups_b));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups_a, Default::default());
		object.created = true;
		let object_id = object.id;
		room.send_command_from_action(object_id, member_2, None, |_| Ok(None)).unwrap_err();
	}

	#[test]
	fn should_send_to_member() {
		let groups = AccessGroups(55);
		let object_template = 5;
		let allow_field_id = 70;

		let template = RoomCreateParams::default();

		let mut room = Room::new(0, template);
		let _member_source_id = room.register_member(MemberCreateParams::stub(groups));
		let member_target_id = room.register_member(MemberCreateParams::stub(groups));

		room.mark_as_attached_in_test(member_target_id).unwrap();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_target_id), groups, Default::default());
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id;

		let commands = vec![S2CCommand::SetLong(LongField {
			object_id,
			field_id: allow_field_id,
			value: 100.into(),
		})];
		room.send_to_member(&member_target_id, &commands).unwrap();

		let out_commands = room.get_member_out_commands_for_test(member_target_id);
		let command = out_commands.get(0);

		assert!(matches!(command, Some(S2CCommand::SetLong(command)) if command.field_id == allow_field_id));
		assert_eq!(out_commands.len(), 1);
	}

	#[test]
	fn should_do_action_not_send_if_object_not_created() {
		let template = RoomCreateParams::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::new(0, template);
		let member_1 = room.register_member(MemberCreateParams::stub(access_groups));
		let member_2 = room.register_member(MemberCreateParams::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups, Default::default());
		let object_id = object.id;
		room.mark_as_attached_in_test(member_1).unwrap();
		room.mark_as_attached_in_test(member_2).unwrap();

		room.send_command_from_action(object_id, member_1, None, |_| Ok(Some(S2CCommand::SetLong(LongField { object_id, field_id: 100, value: 200 }))))
			.unwrap();

		let commands = room.get_member_out_commands_for_test(member_2);
		assert!(commands.is_empty());
	}
}
