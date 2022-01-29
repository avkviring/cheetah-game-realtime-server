use cheetah_matches_relay_common::commands::s2c::S2CCommandWithCreator;
use cheetah_matches_relay_common::constants::GameObjectTemplateId;
use cheetah_matches_relay_common::protocol::commands::output::OutCommand;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup};
use cheetah_matches_relay_common::protocol::frame::channel::ChannelType;
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::object::S2CommandWithFieldInfo;
use crate::room::template::config::Permission;
use crate::room::{Member, Room};

///
/// Методы для отправки команд пользователям
///
///
impl Room {
	pub fn send_to_members<T>(
		&mut self,
		access_groups: AccessGroups,
		object_template: GameObjectTemplateId,
		commands: &[S2CommandWithFieldInfo],
		filter: T,
	) where
		T: Fn(&Member) -> bool,
	{
		#[cfg(test)]
		commands.iter().for_each(|command| {
			self.out_commands.push_front((access_groups, command.command.clone()));
		});

		let channel_type = self
			.current_channel
			.as_ref()
			.unwrap_or(&ChannelType::ReliableSequence(ChannelGroup(0)));

		let current_user = self.current_member_id.unwrap_or(0);

		let permission_manager = self.permission_manager.clone();
		let command_trace_session = self.command_trace_session.clone();
		self.members
			.values_mut()
			.filter(|user| user.attached)
			.filter(|user| user.connected)
			.filter(|user| user.template.groups.contains_any(&access_groups))
			.filter(|user| filter(user))
			.for_each(|user| {
				for command in commands {
					let allow = match command.field {
						None => true,
						Some(field) => {
							permission_manager
								.borrow_mut()
								.get_permission(object_template, field, user.template.groups)
								> Permission::Deny
						}
					};

					if allow {
						command_trace_session
							.borrow_mut()
							.collect_s2c(object_template, user.id, &command.command);

						let command_with_user = S2CCommandWithCreator {
							creator: current_user,
							command: command.command.clone(),
						};

						user.out_commands.push_back(OutCommand {
							channel_type: channel_type.clone(),
							command: BothDirectionCommand::S2CWithCreator(command_with_user),
						});
					}
				}
			});
	}

	pub fn send_to_member(
		&mut self,
		user_id: &RoomMemberId,
		object_template: GameObjectTemplateId,
		commands: &[S2CommandWithFieldInfo],
	) {
		let command_trace_session = self.command_trace_session.clone();
		match self.members.get_mut(user_id) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_id)
			}
			Some(user) => {
				if user.attached && user.connected {
					let groups = user.template.groups;
					for command in commands {
						let allow = match command.field {
							None => true,
							Some(field) => {
								self.permission_manager
									.borrow_mut()
									.get_permission(object_template, field, groups)
									> Permission::Deny
							}
						};
						if allow {
							let channel = self
								.current_channel
								.as_ref()
								.unwrap_or(&ChannelType::ReliableSequence(ChannelGroup(0)));

							let command_with_meta = S2CCommandWithCreator {
								creator: self.current_member_id.unwrap_or(0),
								command: command.command.clone(),
							};
							command_trace_session
								.borrow_mut()
								.collect_s2c(object_template, user.id, &command.command);

							user.out_commands.push_back(OutCommand {
								channel_type: channel.clone(),
								command: BothDirectionCommand::S2CWithCreator(command_with_meta),
							});
						}
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use cheetah_matches_relay_common::commands::types::long::SetLongCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::room::access::AccessGroups;

	use crate::room::object::{Field, S2CommandWithFieldInfo};
	use crate::room::template::config::{Permission, RoomTemplate, UserTemplate};
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
		template
			.permissions
			.set_permission(0, &field_id_2, FieldType::Long, &access_groups, Permission::Rw);

		let mut room = Room::from_template(template);
		let user_1 = room.register_user(UserTemplate::stub(access_groups));
		let user_2 = room.register_user(UserTemplate::stub(access_groups));
		let object = room.create_object(user_1, access_groups);
		object.created = true;
		let object_id = object.id.clone();

		// владельцу разрешены любые операции
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_1,
				field_type: FieldType::Long,
			},
			user_1,
			Permission::Rw,
			None,
			|_| {
				executed = true;
				None
			},
		);
		assert!(executed);

		// RO - по-умолчанию для всех полей
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_1,
				field_type: FieldType::Long,
			},
			user_2,
			Permission::Ro,
			None,
			|_| {
				executed = true;
				None
			},
		);
		assert!(executed);

		// RW - по-умолчанию запрещен
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_1,
				field_type: FieldType::Long,
			},
			user_2,
			Permission::Rw,
			None,
			|_| {
				executed = true;
				None
			},
		);
		assert!(!executed);

		// RW - разрешен для второго поля
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_2,
				field_type: FieldType::Long,
			},
			user_2,
			Permission::Rw,
			None,
			|_| {
				executed = true;
				None
			},
		);
		assert!(executed);
	}

	///
	/// Посылка обратной команды зависит от того изменяют ли поле один клиент или множество
	///
	#[test]
	fn test_build_command_and_send_2() {
		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let field_id_1 = 10;
		let field_id_2 = 20;
		let field_type = FieldType::Long;
		template
			.permissions
			.set_permission(0, &field_id_2, field_type, &access_groups, Permission::Rw);

		let mut room = Room::from_template(template);
		let user_id = room.register_user(UserTemplate::stub(access_groups));
		let object = room.create_object(user_id, access_groups);
		object.access_groups = access_groups;
		object.created = true;
		let object_id = object.id.clone();
		room.mark_as_connected(user_id);

		// изменяем поле, которое никто кроме нас не может изменять
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_1,
				field_type,
			},
			user_id,
			Permission::Rw,
			Option::None,
			|_| {
				executed = true;
				Option::Some(S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: field_id_1,
					value: 0,
				}))
			},
		);
		assert!(executed);
		assert!(room.get_user_out_commands(user_id).is_empty());

		// изменяем поле, которое могут изменять другие пользователи
		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: field_id_2,
				field_type,
			},
			user_id,
			Permission::Rw,
			Option::None,
			|_| {
				executed = true;
				Option::Some(S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: field_id_2,
					value: 0,
				}))
			},
		);
		assert!(executed);
		assert!(matches!(
			room.get_user_out_commands(user_id).get(0),
			Option::Some(S2CCommand::SetLong(_))
		));
	}

	///
	/// Действие не должно выполнится если пользователь не входит в группу объекта
	///
	#[test]
	fn test_build_command_and_send_3() {
		let template = RoomTemplate::default();
		let access_groups_a = AccessGroups(0b01);
		let access_groups_b = AccessGroups(0b10);
		let mut room = Room::from_template(template);
		let user_1 = room.register_user(UserTemplate::stub(access_groups_a));
		let user_2 = room.register_user(UserTemplate::stub(access_groups_b));
		let object = room.create_object(user_1, access_groups_a);
		object.created = true;
		let object_id = object.id.clone();

		let mut executed = false;
		room.do_action_and_send_commands(
			&object_id,
			Field {
				id: 0,
				field_type: FieldType::Long,
			},
			user_2,
			Permission::Ro,
			None,
			|_| {
				executed = true;
				None
			},
		);
		assert!(!executed);
	}

	#[test]
	fn should_send_to_user() {
		let groups = AccessGroups(55);
		let object_template = 5;
		let deny_field_id = 50;
		let allow_field_id = 70;

		let mut template = RoomTemplate::default();
		template
			.permissions
			.set_permission(object_template, &deny_field_id, FieldType::Long, &groups, Permission::Deny);

		let mut room = Room::from_template(template);
		let user_source_id = room.register_user(UserTemplate::stub(groups));
		let user_target_id = room.register_user(UserTemplate::stub(groups));

		room.mark_as_connected(user_target_id);
		let object = room.create_object(user_target_id, groups);
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id.clone();

		let commands = vec![
			S2CommandWithFieldInfo {
				field: Some(Field {
					id: deny_field_id,
					field_type: FieldType::Long,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: deny_field_id,
					value: 0,
				}),
			},
			S2CommandWithFieldInfo {
				field: Some(Field {
					id: allow_field_id,
					field_type: FieldType::Long,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: allow_field_id,
					value: 100,
				}),
			},
		];
		room.current_member_id = Some(user_source_id);
		room.send_to_member(&user_target_id, object_template, &commands);

		let out_commands = room.get_user_out_commands_with_meta(user_target_id);
		let command = out_commands.get(0);

		assert!(
			matches!(command, Some(S2CCommandWithCreator{creator, command: S2CCommand::SetLong
				(command)}) if command.field_id == allow_field_id && *creator == user_source_id)
		);
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
		template.permissions.set_permission(
			object_template,
			&deny_field_id,
			FieldType::Long,
			&access_groups,
			Permission::Deny,
		);

		let mut room = Room::from_template(template);

		let user_1 = room.register_user(UserTemplate::stub(access_groups));
		let user_2 = room.register_user(UserTemplate::stub(access_groups));
		room.mark_as_connected(user_1);
		room.mark_as_connected(user_2);

		let object = room.create_object(user_1, access_groups);
		object.created = true;
		object.template_id = object_template;
		let object_id = object.id.clone();

		let commands = [
			S2CommandWithFieldInfo {
				field: Some(Field {
					id: allow_field_id,
					field_type,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: allow_field_id,
					value: 0,
				}),
			},
			S2CommandWithFieldInfo {
				field: Some(Field {
					id: deny_field_id,
					field_type,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: deny_field_id,
					value: 155,
				}),
			},
		];

		room.send_to_members(access_groups, object_template, &commands, |_| true);

		let commands = room.get_user_out_commands(user_2);
		assert!(matches!(commands.get(0),Option::Some(S2CCommand::SetLong(c)) if c.field_id == allow_field_id));
		assert_eq!(commands.len(), 1);
	}

	#[test]
	fn should_do_action_not_send_if_object_not_created() {
		let field_id = 10;
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(55);
		let mut room = Room::from_template(template);
		let user_1 = room.register_user(UserTemplate::stub(access_groups));
		let user_2 = room.register_user(UserTemplate::stub(access_groups));
		let object = room.create_object(user_1, access_groups);
		let object_id = object.id.clone();
		room.mark_as_connected(user_1);
		room.mark_as_connected(user_2);

		room.do_action_and_send_commands(
			&object_id.clone(),
			Field {
				id: field_id,
				field_type: FieldType::Long,
			},
			user_1,
			Permission::Rw,
			Option::None,
			|_| {
				Option::Some(S2CCommand::SetLong(SetLongCommand {
					object_id,
					field_id: 100,
					value: 200,
				}))
			},
		);

		let commands = room.get_user_out_commands(user_2);
		assert!(commands.is_empty());
	}
}
