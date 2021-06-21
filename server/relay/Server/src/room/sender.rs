use std::slice::Iter;

use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::{S2CCommand, S2CCommandWithMeta};
use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserId;

use crate::room::object::{FieldIdAndType, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::{Room, User};

impl Room {
	///
	/// Проверить права доступа на изменения объекта, если права достаточные - создать команду и отправить всем пользователям,
	/// с проверкой прав доступа при отправке
	///
	pub fn build_command_and_send<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field_id: &FieldId,
		field_type: FieldType,
		command_owner_user: UserId,
		permission: Permission,
		out_command_builder: T,
	) where
		T: FnOnce(&mut GameObject) -> Option<S2CCommand>,
	{
		self.do_build_command_and_send(
			game_object_id,
			field_id,
			field_type,
			command_owner_user,
			permission,
			None,
			out_command_builder,
		);
	}

	///
	/// Проверить права доступа на изменение объекта, если прав достаточно - создать команду и отправить заданному пользователю
	///
	pub fn build_command_and_send_to_user<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field_id: &FieldId,
		field_type: FieldType,
		command_owner_user: UserId,
		permission: Permission,
		out_command_builder: T,
		target_user: UserId,
	) where
		T: FnOnce(&mut GameObject) -> Option<S2CCommand>,
	{
		self.do_build_command_and_send(
			game_object_id,
			field_id,
			field_type,
			command_owner_user,
			permission,
			Some(target_user),
			out_command_builder,
		);
	}

	///
	/// Проверить права доступа, выполнить действие, результат выполнения отправить клиентам (клиенту)
	///
	/// - владелец объекта получает обновления если только данные доступны на запись другим клиентам
	/// - владелец объекта имеет полный доступ к полям объекта, информация о правах игнорируется
	///
	fn do_build_command_and_send<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field_id: &FieldId,
		field_type: FieldType,
		command_owner_user: UserId,
		permission: Permission,
		target_user: Option<UserId>,
		out_command_builder: T,
	) where
		T: FnOnce(&mut GameObject) -> Option<S2CCommand>,
	{
		let room_id = self.id;

		let permission_manager = self.permission_manager.clone();

		let current_user_access_group = match self.users.get(&command_owner_user) {
			None => {
				log::error!("[room({})] user({}) not found", self.id, command_owner_user);
				return;
			}
			Some(user) => user.template.access_groups.clone(),
		};

		if let Some(object) = self.get_object_mut(&game_object_id) {
			// проверяем группу доступа
			if !object.access_groups.contains_any(&current_user_access_group) {
				log::error!(
					"[room({})] user({}) group({:?}) don't allow access to object ({:?})",
					room_id,
					command_owner_user,
					current_user_access_group,
					object.access_groups
				);
				return;
			}

			let object_owner = if let ObjectOwner::User(owner) = object.id.owner {
				Option::Some(owner)
			} else {
				Option::None
			};

			let current_user_is_object_owner = object_owner == Option::Some(command_owner_user);
			let allow = current_user_is_object_owner
				|| permission_manager
					.borrow_mut()
					.get_permission(object.template, *field_id, field_type, current_user_access_group)
					>= permission;

			if !allow {
				log::error!(
					"[room({:?})] user({:?}) has not permissions({:?}) for action with object({:?}), field({:?}), field_type({:?})",
					self.id,
					command_owner_user,
					permission,
					game_object_id,
					field_id,
					field_type
				);
				return;
			}

			let command = out_command_builder(object);
			// отправляем команду только для созданного объекта
			if object.created {
				let groups = object.access_groups.clone();
				let template = object.template;

				if let Some(command) = command {
					let commands_with_field = S2CommandWithFieldInfo {
						field: Some(FieldIdAndType {
							field_id: *field_id,
							field_type,
						}),
						command,
					};
					let commands = [commands_with_field];

					if let Some(target_user) = target_user {
						self.send_to_user(&target_user, template, commands.iter());
					} else {
						self.send(groups, template, commands.iter(), |user| {
							let mut permission_manager = permission_manager.borrow_mut();
							if object_owner == Option::Some(user.template.id) {
								permission_manager.has_write_access(template, *field_id, field_type)
							} else {
								true
							}
						});
					}
				};
			}
		} else {
			log::error!("room[({:?})] do_action object not found ({:?}) ", self.id, game_object_id);
		}
	}

	pub fn send<T>(&mut self, access_groups: AccessGroups, template: GameObjectTemplateId, commands: Iter<S2CommandWithFieldInfo>, filter: T)
	where
		T: Fn(&User) -> bool,
	{
		#[cfg(test)]
		commands.clone().for_each(|command| {
			self.out_commands.push_front((access_groups, command.command.clone()));
		});

		let channel_type = self
			.current_channel
			.as_ref()
			.unwrap_or(&ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let meta = match &self.current_user {
			None => S2CMetaCommandInformation::new(0, &C2SMetaCommandInformation::default()),
			Some(user) => S2CMetaCommandInformation::new(user.clone(), self.current_meta.as_ref().unwrap_or(&C2SMetaCommandInformation::default())),
		};
		let room_id = self.id;
		let tracer = self.tracer.clone();

		let permission_manager = self.permission_manager.clone();
		self.users
			.values_mut()
			.filter(|user| user.attached)
			.filter(|user| user.protocol.is_some())
			.filter(|user| user.template.access_groups.contains_any(&access_groups))
			.filter(|user| filter(user))
			.for_each(|user| {
				let protocol = user.protocol.as_mut().unwrap();
				for command in commands.clone() {
					let allow = match &command.field {
						None => true,
						Some(FieldIdAndType { field_id, field_type }) => {
							permission_manager
								.borrow_mut()
								.get_permission(template, *field_id, *field_type, user.template.access_groups)
								> Permission::Deny
						}
					};

					if allow {
						let command_with_meta = S2CCommandWithMeta {
							meta: meta.clone(),
							command: command.command.clone(),
						};
						tracer.on_s2c_command(room_id, user.template.id.clone(), &command_with_meta);
						let application_command = ApplicationCommand::S2CCommandWithMeta(command_with_meta);
						protocol
							.out_commands_collector
							.add_command(channel_type.clone(), application_command.clone());
					}
				}
			});
	}

	pub fn send_to_user(&mut self, user_id: &UserId, object_template: GameObjectTemplateId, commands: Iter<S2CommandWithFieldInfo>) {
		match self.users.get_mut(user_id) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_id)
			}
			Some(user) => {
				if let Some(ref mut protocol) = user.protocol {
					if user.attached {
						let groups = user.template.access_groups;
						for command in commands {
							let allow = match command.field {
								None => true,
								Some(FieldIdAndType { field_id, field_type }) => {
									self.permission_manager
										.borrow_mut()
										.get_permission(object_template, field_id, field_type, groups)
										> Permission::Deny
								}
							};
							if allow {
								let default = C2SMetaCommandInformation::default();
								let meta = self.current_meta.as_ref().unwrap_or(&default);
								let channel = self
									.current_channel
									.as_ref()
									.unwrap_or(&ApplicationCommandChannelType::ReliableSequenceByGroup(0));

								let command_with_meta = S2CCommandWithMeta {
									meta: S2CMetaCommandInformation::new(self.current_user.unwrap_or(0), meta),
									command: command.command.clone(),
								};
								self.tracer.on_s2c_command(self.id, user.template.id, &command_with_meta);
								let application_command = ApplicationCommand::S2CCommandWithMeta(command_with_meta);
								protocol.out_commands_collector.add_command(channel.clone(), application_command.clone());
							}
						}
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::long::SetLongCommand;
	use cheetah_relay_common::commands::command::{S2CCommand, S2CCommandWithMeta};
	use cheetah_relay_common::room::access::AccessGroups;

	use crate::room::object::{FieldIdAndType, S2CommandWithFieldInfo};
	use crate::room::template::config::{Permission, RoomTemplate, UserTemplate};
	use crate::room::types::FieldType;
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
		let user_1 = 1;
		let user_2 = 2;
		room.register_user(UserTemplate::stub(user_1, access_groups));
		room.register_user(UserTemplate::stub(user_2, access_groups));
		let object = room.create_object(user_1, access_groups);
		object.created = true;
		let object_id = object.id.clone();

		// владельцу разрешены любые операции
		let mut executed = false;
		room.do_build_command_and_send(&object_id, &field_id_1, FieldType::Long, user_1, Permission::Rw, None, |_| {
			executed = true;
			None
		});
		assert!(executed);

		// RO - по-умолчанию для всех полей
		let mut executed = false;
		room.do_build_command_and_send(&object_id, &field_id_1, FieldType::Long, user_2, Permission::Ro, None, |_| {
			executed = true;
			None
		});
		assert!(executed);

		// RW - по-умолчанию запрещен
		let mut executed = false;
		room.do_build_command_and_send(&object_id, &field_id_1, FieldType::Long, user_2, Permission::Rw, None, |_| {
			executed = true;
			None
		});
		assert!(!executed);

		// RW - разрешен для второго поля
		let mut executed = false;
		room.do_build_command_and_send(&object_id, &field_id_2, FieldType::Long, user_2, Permission::Rw, None, |_| {
			executed = true;
			None
		});
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
		let user = 1;
		room.register_user(UserTemplate::stub(user, access_groups));
		let object = room.create_object(user, access_groups);
		object.access_groups = access_groups.clone();
		object.created = true;
		let object_id = object.id.clone();
		room.mark_as_connected(user);

		// изменяем поле, которое никто кроме нас не может изменять
		let mut executed = false;
		room.build_command_and_send(&object_id, &field_id_1, field_type, user, Permission::Rw, |_| {
			executed = true;
			Option::Some(S2CCommand::SetLong(SetLongCommand {
				object_id: object_id.clone(),
				field_id: field_id_1,
				value: 0,
			}))
		});
		assert!(executed);
		assert!(room.get_user_out_commands(user).is_empty());

		// изменяем поле, которое могут изменять другие пользователи
		let mut executed = false;
		room.build_command_and_send(&object_id, &field_id_2, field_type, user, Permission::Rw, |_| {
			executed = true;
			Option::Some(S2CCommand::SetLong(SetLongCommand {
				object_id: object_id.clone(),
				field_id: field_id_2,
				value: 0,
			}))
		});
		assert!(executed);
		assert!(matches!(room.get_user_out_commands(user).get(0), Option::Some(S2CCommand::SetLong(_))));
	}

	///
	/// Действие не должно выполнится если пользователь не входит в группу объекта
	///
	#[test]
	fn test_build_command_and_send_3() {
		let mut template = RoomTemplate::default();
		let access_groups_a = AccessGroups(0b01);
		let access_groups_b = AccessGroups(0b10);
		let mut room = Room::from_template(template);
		let user_1 = 1;
		let user_2 = 2;
		room.register_user(UserTemplate::stub(user_1, access_groups_a));
		room.register_user(UserTemplate::stub(user_2, access_groups_b));
		let object = room.create_object(user_1, access_groups_a);
		object.created = true;
		let object_id = object.id.clone();

		let mut executed = false;
		room.do_build_command_and_send(&object_id, &0, FieldType::Long, user_2, Permission::Ro, None, |_| {
			executed = true;
			None
		});
		assert!(!executed);
	}

	#[test]
	fn should_send_to_user() {
		let user_source_id = 9;
		let user_target_id = 10;
		let groups = AccessGroups(55);
		let object_template = 5;
		let deny_field_id = 50;
		let allow_field_id = 70;

		let mut template = RoomTemplate::default();
		template
			.permissions
			.set_permission(object_template, &deny_field_id, FieldType::Long, &groups, Permission::Deny);

		let mut room = Room::from_template(template);
		room.register_user(UserTemplate::stub(user_target_id, groups));
		room.register_user(UserTemplate::stub(user_source_id, groups));

		room.mark_as_connected(user_target_id);
		let object = room.create_object(user_target_id, groups);
		object.created = true;
		object.template = object_template;
		let object_id = object.id.clone();

		let mut commands = Vec::new();
		commands.push(S2CommandWithFieldInfo {
			field: Some(FieldIdAndType {
				field_id: deny_field_id,
				field_type: FieldType::Long,
			}),
			command: S2CCommand::SetLong(SetLongCommand {
				object_id: object_id.clone(),
				field_id: deny_field_id,
				value: 0,
			}),
		});
		commands.push(S2CommandWithFieldInfo {
			field: Some(FieldIdAndType {
				field_id: allow_field_id,
				field_type: FieldType::Long,
			}),
			command: S2CCommand::SetLong(SetLongCommand {
				object_id: object_id.clone(),
				field_id: allow_field_id,
				value: 100,
			}),
		});
		room.current_user = Some(user_source_id);
		room.send_to_user(&user_target_id, object_template, commands.iter());

		let out_commands = room.get_user_out_commands_with_meta(user_target_id);
		let command = out_commands.get(0);

		assert!(
			matches!(command, Some(S2CCommandWithMeta{meta, command: S2CCommand::SetLong(command)}) if command.field_id == allow_field_id && meta.user_id == user_source_id)
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
		let user_1 = 1;
		let user_2 = 2;

		let allow_field_id = 10;
		let deny_field_id = 11;
		let field_type = FieldType::Long;

		let mut template = RoomTemplate::default();
		template
			.permissions
			.set_permission(object_template, &deny_field_id, FieldType::Long, &access_groups, Permission::Deny);

		let mut room = Room::from_template(template);
		room.register_user(UserTemplate::stub(user_1, access_groups));
		room.register_user(UserTemplate::stub(user_2, access_groups));
		room.mark_as_connected(user_1);
		room.mark_as_connected(user_2);

		let object = room.create_object(user_1, access_groups);
		object.created = true;
		object.template = object_template;
		let object_id = object.id.clone();

		let commands = [
			S2CommandWithFieldInfo {
				field: Some(FieldIdAndType {
					field_id: allow_field_id,
					field_type,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: allow_field_id,
					value: 0,
				}),
			},
			S2CommandWithFieldInfo {
				field: Some(FieldIdAndType {
					field_id: deny_field_id,
					field_type,
				}),
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: object_id.clone(),
					field_id: deny_field_id,
					value: 155,
				}),
			},
		];

		room.send(access_groups, object_template, commands.iter(), |_| true);

		let commands = room.get_user_out_commands(user_2);
		assert!(matches!(commands.get(0),Option::Some(S2CCommand::SetLong(c)) if c.field_id == allow_field_id));
		assert_eq!(commands.len(), 1);
	}

	#[test]
	fn should_do_action_not_send_if_object_not_created() {
		let user_1 = 1;
		let user_2 = 2;
		let field_id = 10;

		let mut template = RoomTemplate::default();
		let access_groups = AccessGroups(55);

		let mut room = Room::from_template(template);
		room.register_user(UserTemplate::stub(user_1, access_groups));
		room.register_user(UserTemplate::stub(user_2, access_groups));

		let object = room.create_object(user_1, access_groups);
		let object_id = object.id.clone();
		room.mark_as_connected(user_1);
		room.mark_as_connected(user_2);

		room.build_command_and_send(&object_id.clone(), &field_id, FieldType::Long, user_1, Permission::Rw, |_| {
			Option::Some(S2CCommand::SetLong(SetLongCommand {
				object_id,
				field_id: 100,
				value: 200,
			}))
		});

		let commands = room.get_user_out_commands(user_2);
		assert!(commands.is_empty());
	}
}
