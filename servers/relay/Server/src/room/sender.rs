use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::{S2CCommand, S2CCommandWithMeta};
use cheetah_relay_common::constants::FieldId;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::object::GameObject;
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::{Room, User};

#[cfg(test)]
use std::collections::VecDeque;

impl Room {
	pub fn send_object_to_group(&mut self, object: &GameObject) {
		let mut commands = Vec::new();
		object.collect_create_commands(&mut commands);
		commands.into_iter().for_each(|c| {
			self.send_to_group(object.access_groups, c, |_| true);
		})
	}

	///
	/// Проверить права доступа, выполнить действие, результат выполнения отправить клиентам
	///
	/// - владелец объекта получает обновления если только данные доступны на запись другим клиентам
	/// - владелец объекта имеет полный доступ к полям объекта, информация о правах игнорируется
	///
	pub fn do_action<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field_id: &FieldId,
		field_type: FieldType,
		command_owner_user: &UserPublicKey,
		permission: Permission,
		action: T,
	) where
		T: FnOnce(&mut GameObject) -> Option<S2CCommand>,
	{
		let permission_manager = self.permission_manager.clone();

		let current_user_access_group = match self.users.get(command_owner_user) {
			None => {
				log::error!("[room({})] user({}) not found", self.id, command_owner_user);
				#[cfg(test)]
				panic!("[room({})] user({}) not found", self.id, command_owner_user);
				#[cfg(not(test))]
				return;
			}
			Some(user) => user.template.access_groups.clone(),
		};

		if let Some(object) = self.get_object_mut(&game_object_id) {
			let object_owner = if let ObjectOwner::User(owner) = object.id.owner {
				Option::Some(owner)
			} else {
				Option::None
			};

			let current_user_is_object_owner = object_owner == Option::Some(*command_owner_user);
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

			let command = action(object);
			let groups = object.access_groups.clone();
			let template = object.template;

			if let Some(command) = command {
				self.send_to_group(groups, command, |user| {
					let mut permission_manager = permission_manager.borrow_mut();
					if object_owner == Option::Some(user.template.public_key) {
						permission_manager.has_write_access(template, *field_id, field_type)
					} else {
						permission_manager.get_permission(template, *field_id, field_type, user.template.access_groups) > Permission::Deny
					}
				});
			};
		};
	}

	pub fn send_to_group<T>(&mut self, access_groups: AccessGroups, command: S2CCommand, filter: T)
	where
		T: Fn(&User) -> bool,
	{
		#[cfg(test)]
		self.out_commands.push_front((access_groups, command.clone()));

		let channel_type = self
			.current_channel
			.as_ref()
			.unwrap_or(&ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let meta = match &self.current_user {
			None => S2CMetaCommandInformation::new(0, &C2SMetaCommandInformation { timestamp: 0 }),
			Some(user) => S2CMetaCommandInformation::new(
				user.clone(),
				self.current_meta.as_ref().unwrap_or(&C2SMetaCommandInformation { timestamp: 0 }),
			),
		};

		let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
			meta,
			command: command.clone(),
		});

		let room_id = self.id;
		let tracer = self.tracer.clone();
		self.users
			.values_mut()
			.filter(|user| user.attached)
			.filter(|user| user.protocol.is_some())
			.filter(|user| user.template.access_groups.contains_any(&access_groups))
			.filter(|user| filter(user))
			.for_each(|user| {
				let protocol = user.protocol.as_mut().unwrap();
				tracer.on_s2c_command(room_id, user.template.public_key.clone(), &command);
				protocol
					.out_commands_collector
					.add_command(channel_type.clone(), application_command.clone())
			});
	}

	pub fn send_to_user(&mut self, user_public_key: &u32, commands: Vec<S2CCommand>) {
		#[cfg(test)]
		{
			let user_commands = self.out_commands_by_users.entry(user_public_key.clone()).or_insert(VecDeque::new());
			for command in &commands {
				user_commands.push_front(command.clone());
			}
		}

		match self.users.get_mut(user_public_key) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_public_key)
			}
			Some(user) => {
				if let Some(ref mut protocol) = user.protocol {
					if user.attached {
						for command in commands {
							self.tracer.on_s2c_command(self.id, user.template.public_key, &command);
							let meta = self.current_meta.as_ref().unwrap();
							let channel = self.current_channel.as_ref().unwrap();
							let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
								meta: S2CMetaCommandInformation::new(user_public_key.clone(), meta),
								command,
							});
							protocol.out_commands_collector.add_command(channel.clone(), application_command.clone());
						}
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;

	use cheetah_relay_common::commands::command::event::EventCommand;
	use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::protocol::frame::applications::ApplicationCommandChannelType;
	use cheetah_relay_common::protocol::relay::RelayProtocol;

	use crate::room::tests::create_template;
	use crate::room::Room;

	#[test]
	fn should_send_command_to_other_user() {
		let (template, user_template) = create_template();
		let mut room = Room::new_with_template(template);
		room.current_user.replace(user_template.public_key + 1); // команда пришла от другого пользователя
		room.current_meta.replace(C2SMetaCommandInformation { timestamp: 0 });
		room.current_channel.replace(ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let user = room.get_user_mut(&user_template.public_key).unwrap();
		user.attached = true;
		user.protocol.replace(RelayProtocol::new(&Instant::now()));

		room.send_to_group(
			user_template.access_groups.clone(),
			S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			}),
			|_| true,
		);

		let user = room.get_user(&user_template.public_key).unwrap();
		let protocol = user.protocol.as_ref().unwrap();
		assert_eq!(protocol.out_commands_collector.commands.reliable.len(), 1);
	}
}
