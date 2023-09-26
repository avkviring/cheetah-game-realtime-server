use cheetah_common::commands::guarantees::{ChannelGroup, ReliabilityGuarantees};
use cheetah_common::commands::s2c::{S2CCommandWithCreator, S2CCommandWithMeta};
use cheetah_common::commands::{BothDirectionCommand, CommandWithChannelType};
use cheetah_common::room::access::AccessGroups;
use cheetah_game_realtime_protocol::RoomMemberId;
use crate::server::room::command::ServerCommandError;
use crate::server::room::member::RoomMember;
use crate::server::room::Room;

///
/// Методы для отправки команд пользователям
///
impl Room {
	pub fn send_to_members<T>(&mut self, access_groups: AccessGroups, commands: &[S2CCommandWithMeta], filter: T) -> Result<(), ServerCommandError>
	where
		T: Fn(&RoomMember) -> bool,
	{
		#[cfg(test)]
		for command in commands.iter() {
			self.test_out_commands.push_front((access_groups, command.command.clone()));
		}

		let channel_type = self.current_channel.as_ref().unwrap_or(&ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let members_for_send = self
			.members
			.values_mut()
			.filter(|member| member.attached)
			.filter(|member| member.connected)
			.filter(|member| member.template.groups.contains_any(&access_groups))
			.filter(|member| filter(member));

		for member in members_for_send {
			commands.iter().for_each(|command| {
				let member_with_creator = S2CCommandWithCreator {
					creator: command.creator,
					command: command.command.clone(),
				};
				member.out_commands.push(CommandWithChannelType {
					channel_type: *channel_type,
					command: BothDirectionCommand::S2CWithCreator(member_with_creator),
				});
			});
		}

		Ok(())
	}

	pub fn send_to_member(&mut self, member_id: &RoomMemberId, commands: &[S2CCommandWithMeta]) -> Result<(), ServerCommandError> {
		let channel = self.current_channel.unwrap_or(ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let member = self.get_member_mut(member_id)?;

		if member.attached && member.connected {
			for command in commands {
				let command_with_meta = S2CCommandWithCreator {
					creator: command.creator,
					command: command.command.clone(),
				};
				let member = self.get_member_mut(member_id)?;
				member.out_commands.push(CommandWithChannelType {
					channel_type: channel,
					command: BothDirectionCommand::S2CWithCreator(command_with_meta),
				});
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithCreator, S2CCommandWithMeta};
	use cheetah_common::commands::types::long::SetLongCommand;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::field::{Field, FieldType};
	use cheetah_common::room::owner::GameObjectOwner;
	use crate::server::room::Room;
	use crate::server::room::template::config::{MemberTemplate, RoomTemplate};


	///
	/// Не посылаем обратную команду, тому кто ее вызвал
	///
	#[test]
	fn should_dont_sent_command_back() {
		let access_groups = AccessGroups(55);
		let field = Field { id: 10, field_type: FieldType::Long };

		let mut room = Room::from_template(RoomTemplate::default());
		let member_id = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_id), access_groups);
		object.access_groups = access_groups;
		object.created = true;
		let object_id = object.id;
		room.mark_as_connected_in_test(member_id).unwrap();

		room.send_command_from_action(object_id, field, member_id, None, |_| {
			Ok(Some(S2CCommand::SetLong(SetLongCommand {
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
		let template = RoomTemplate::default();
		let access_groups_a = AccessGroups(0b01);
		let access_groups_b = AccessGroups(0b10);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups_a));
		let member_2 = room.register_member(MemberTemplate::stub(access_groups_b));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups_a);
		object.created = true;
		let object_id = object.id;
		room.send_command_from_action(object_id, Field { id: 0, field_type: FieldType::Long }, member_2, None, |_| Ok(None))
			.unwrap_err();
	}

	#[test]
	fn should_send_to_member() {
		let groups = AccessGroups(55);
		let object_template = 5;
		let allow_field_id = 70;

		let template = RoomTemplate::default();

		let mut room = Room::from_template(template);
		let _member_source_id = room.register_member(MemberTemplate::stub(groups));
		let member_target_id = room.register_member(MemberTemplate::stub(groups));

		room.mark_as_connected_in_test(member_target_id).unwrap();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_target_id), groups);
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id;

		let commands = vec![S2CCommandWithMeta {
			field: Some(Field {
				id: allow_field_id,
				field_type: FieldType::Long,
			}),
			creator: u16::MAX,
			command: S2CCommand::SetLong(SetLongCommand {
				object_id,
				field_id: allow_field_id,
				value: 100.into(),
			}),
		}];
		room.send_to_member(&member_target_id, &commands).unwrap();

		let out_commands = room.test_get_member_out_commands_with_meta(member_target_id);
		let command = out_commands.get(0);

		assert!(matches!(command, Some(S2CCommandWithCreator{creator: _member_source_id, command:
				S2CCommand::SetLong(command)}) if command.field_id == allow_field_id));
		assert_eq!(out_commands.len(), 1);
	}

	#[test]
	fn should_do_action_not_send_if_object_not_created() {
		let field_id = 10;
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		let member_2 = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups);
		let object_id = object.id;
		room.mark_as_connected_in_test(member_1).unwrap();
		room.mark_as_connected_in_test(member_2).unwrap();

		room.send_command_from_action(
			object_id,
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			member_1,
			None,
			|_| Ok(Some(S2CCommand::SetLong(SetLongCommand { object_id, field_id: 100, value: 200 }))),
		)
		.unwrap();

		let commands = room.get_member_out_commands_for_test(member_2);
		assert!(commands.is_empty());
	}
}
