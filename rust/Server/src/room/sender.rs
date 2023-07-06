use cheetah_common::commands::guarantees::{ChannelGroup, ReliabilityGuarantees};
use cheetah_common::commands::{BothDirectionCommand, CommandWithChannelType};
use std::rc::Rc;

use cheetah_common::commands::s2c::{S2CCommandWithCreator, S2CCommandWithMeta};
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectTemplateId;
use cheetah_protocol::RoomMemberId;

use crate::room::command::ServerCommandError;
use crate::room::member::RoomMember;
use crate::room::template::config::Permission;
use crate::room::Room;

///
/// Методы для отправки команд пользователям
///
impl Room {
	pub fn send_to_members<T>(&mut self, access_groups: AccessGroups, object_template: Option<GameObjectTemplateId>, commands: &[S2CCommandWithMeta], filter: T) -> Result<(), ServerCommandError>
	where
		T: Fn(&RoomMember) -> bool,
	{
		#[cfg(test)]
		for command in commands.iter() {
			self.test_out_commands.push_front((access_groups, command.command.clone()));
		}

		let channel_type = self.current_channel.as_ref().unwrap_or(&ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));

		let permission_manager = Rc::clone(&self.permission_manager);
		let members_for_send = self
			.members
			.values_mut()
			.filter(|member| member.attached)
			.filter(|member| member.connected)
			.filter(|member| member.template.groups.contains_any(&access_groups))
			.filter(|member| filter(member));

		for member in members_for_send {
			commands
				.iter()
				.filter(|&command| {
					if let Some(template) = object_template {
						match command.field {
							None => true,
							Some(field) => permission_manager.borrow_mut().get_permission(template, field, member.template.groups) > Permission::Deny,
						}
					} else {
						true
					}
				})
				.for_each(|command| {
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

	pub fn send_to_member(&mut self, member_id: &RoomMemberId, object_template: GameObjectTemplateId, commands: &[S2CCommandWithMeta]) -> Result<(), ServerCommandError> {
		let permission_manager = Rc::clone(&self.permission_manager);
		let channel = self.current_channel.unwrap_or(ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let member = self.get_member_mut(member_id)?;

		if member.attached && member.connected {
			let groups = member.template.groups;
			for command in commands {
				let allow = match command.field {
					None => true,
					Some(field) => permission_manager.borrow_mut().get_permission(object_template, field, groups) > Permission::Deny,
				};
				if allow {
					let command_with_meta = S2CCommandWithCreator {
						creator: command.creator,
						command: command.command.clone(),
					};
					member.out_commands.push(CommandWithChannelType {
						channel_type: channel,
						command: BothDirectionCommand::S2CWithCreator(command_with_meta),
					});
				}
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

	use crate::room::template::config::{MemberTemplate, Permission, RoomTemplate};
	use crate::room::Room;

	///
	/// Проверяем проверку прав доступа на изменения данных объекта
	///
	#[test]
	fn test_build_command_and_send_1() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let field_id_1 = 10;
		let field_id_2 = 11;
		template.permissions.set_permission(0, &field_id_2, FieldType::Long, &access_groups, Permission::Rw);

		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		let member_2 = room.register_member(MemberTemplate::stub(access_groups));
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups);
		object.created = true;
		let object_id = object.id;

		// владельцу разрешены любые операции
		room.send_command_from_action(
			object_id,
			Field {
				id: field_id_1,
				field_type: FieldType::Long,
			},
			member_1,
			Permission::Rw,
			None,
			|_| Ok(None),
		)
		.unwrap();

		// RO - по-умолчанию для всех полей
		room.send_command_from_action(
			object_id,
			Field {
				id: field_id_1,
				field_type: FieldType::Long,
			},
			member_2,
			Permission::Rw,
			None,
			|_| Ok(None),
		)
		.unwrap();
	}

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

		room.send_command_from_action(object_id, field, member_id, Permission::Rw, None, |_| {
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
		room.send_command_from_action(object_id, Field { id: 0, field_type: FieldType::Long }, member_2, Permission::Ro, None, |_| Ok(None))
			.unwrap_err();
	}

	#[test]
	fn should_send_to_member() {
		let groups = AccessGroups(55);
		let object_template = 5;
		let deny_field_id = 50;
		let allow_field_id = 70;

		let mut template = RoomTemplate::default();
		template.permissions.set_permission(object_template, &deny_field_id, FieldType::Long, &groups, Permission::Deny);

		let mut room = Room::from_template(template);
		let _member_source_id = room.register_member(MemberTemplate::stub(groups));
		let member_target_id = room.register_member(MemberTemplate::stub(groups));

		room.mark_as_connected_in_test(member_target_id).unwrap();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_target_id), groups);
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id;

		let commands = vec![
			S2CCommandWithMeta {
				field: Some(Field {
					id: deny_field_id,
					field_type: FieldType::Long,
				}),
				creator: u16::MAX,
				command: S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: deny_field_id,
					value: 0,
				}),
			},
			S2CCommandWithMeta {
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
			},
		];
		room.send_to_member(&member_target_id, object_template, &commands).unwrap();

		let out_commands = room.test_get_member_out_commands_with_meta(member_target_id);
		let command = out_commands.get(0);

		assert!(matches!(command, Some(S2CCommandWithCreator{creator: _member_source_id, command:
				S2CCommand::SetLong(command)}) if command.field_id == allow_field_id));
		assert_eq!(out_commands.len(), 1);
	}

	///
	/// Не посылать обновление клиенту если это запрещено правами
	///
	#[test]
	fn should_send_with_permission() {
		let access_groups = AccessGroups(0b111);
		let object_template = 100;
		let allow_field_id = 10;
		let deny_field_id = 11;
		let field_type = FieldType::Long;

		let mut template = RoomTemplate::default();
		template.permissions.set_permission(object_template, &deny_field_id, FieldType::Long, &access_groups, Permission::Deny);

		let mut room = Room::from_template(template);

		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		let member_2 = room.register_member(MemberTemplate::stub(access_groups));
		room.mark_as_connected_in_test(member_1).unwrap();
		room.mark_as_connected_in_test(member_2).unwrap();

		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Member(member_1), access_groups);
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id;

		let commands = [
			S2CCommandWithMeta {
				field: Some(Field { id: allow_field_id, field_type }),
				creator: u16::MAX,
				command: S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: allow_field_id,
					value: 0,
				}),
			},
			S2CCommandWithMeta {
				field: Some(Field { id: deny_field_id, field_type }),
				creator: u16::MAX,
				command: S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: deny_field_id,
					value: 155,
				}),
			},
		];

		room.send_to_members(access_groups, Some(object_template), &commands, |_| true).unwrap();

		let commands = room.get_member_out_commands_for_test(member_2);
		assert!(matches!(commands.get(0),Some(S2CCommand::SetLong(c)) if c.field_id == allow_field_id));
		assert_eq!(commands.len(), 1);
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
			Permission::Rw,
			None,
			|_| Ok(Some(S2CCommand::SetLong(SetLongCommand { object_id, field_id: 100, value: 200 }))),
		)
		.unwrap();

		let commands = room.get_member_out_commands_for_test(member_2);
		assert!(commands.is_empty());
	}
}
