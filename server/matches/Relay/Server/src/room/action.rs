use cheetah_matches_relay_common::commands::command::S2CCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::ObjectOwner;
use cheetah_matches_relay_common::room::UserId;

use crate::room::object::{FieldIdAndType, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
use crate::room::types::FieldType;
use crate::room::Room;

///
/// Выполнение действий по изменению данных игровых объектов с проверкой прав доступа и отсылки
/// результата клиентам
///
impl Room {
	///
	/// Проверить права доступа, выполнить действие, результат выполнения отправить клиентам (клиенту)
	///
	/// - владелец объекта получает обновления если только данные доступны на запись другим клиентам
	/// - владелец объекта имеет полный доступ к полям объекта, информация о правах игнорируется
	///
	pub fn change_data_and_send<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field_id: &FieldId,
		field_type: FieldType,
		command_owner_user: UserId,
		permission: Permission,
		target_user: Option<UserId>,
		action: T,
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
			Some(user) => user.template.groups.clone(),
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

			let command = action(object);
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

					match target_user {
						Some(target_user) => {
							self.send_to_user(&target_user, template, commands.iter());
						}
						None => {
							self.send_to_users(groups, template, commands.iter(), |user| {
								let mut permission_manager = permission_manager.borrow_mut();
								// отправляем себе только если есть права на запись
								// иначе никто другой не может вносит изменения в данное поле и
								// отправлять себе как единственному источнику изменений избыточно
								if object_owner == Option::Some(user.id) {
									permission_manager.has_write_access(template, *field_id, field_type)
								} else {
									true
								}
							});
						}
					}
				};
			}
		} else {
			log::error!("room[({:?})] do_action object not found ({:?}) ", self.id, game_object_id);
		}
	}
}
