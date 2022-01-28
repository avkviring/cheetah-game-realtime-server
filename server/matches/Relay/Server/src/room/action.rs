use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::object::{Field, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::Permission;
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
	pub fn do_action_and_send_commands<T>(
		&mut self,
		game_object_id: &GameObjectId,
		field: Field,
		creator_id: RoomMemberId,
		permission: Permission,
		target: Option<RoomMemberId>,
		action: T,
	) where
		T: FnOnce(&mut GameObject) -> Option<S2CCommand>,
	{
		let room_id = self.id;
		let permission_manager = self.permission_manager.clone();
		let creator_access_group = match self.members.get(&creator_id) {
			None => {
				log::error!("[room({})] user({}) not found", self.id, creator_id);
				return;
			}
			Some(member) => member.template.groups,
		};

		if let Some(object) = self.get_object_mut(game_object_id) {
			// проверяем группу доступа
			if !object.access_groups.contains_any(&creator_access_group) {
				log::error!(
					"[room({})] user({}) group({:?}) don't allow access to object ({:?})",
					room_id,
					creator_id,
					creator_access_group,
					object.access_groups
				);
				return;
			}

			let object_owner = if let GameObjectOwner::User(owner) = object.id.owner {
				Option::Some(owner)
			} else {
				Option::None
			};

			let is_creator_object_owner = object_owner == Option::Some(creator_id);

			let allow = is_creator_object_owner
				|| permission_manager
					.borrow_mut()
					.get_permission(object.template, field, creator_access_group)
					>= permission;

			if !allow {
				log::error!(
					"[room({:?})] user({:?}) has not permissions({:?}) for action with object({:?}), field({:?})",
					self.id,
					creator_id,
					permission,
					game_object_id,
					field
				);
				return;
			}

			let command = action(object);
			// отправляем команду только для созданного объекта
			if object.created {
				let groups = object.access_groups;
				let template = object.template;

				if let Some(command) = command {
					let commands_with_field = S2CommandWithFieldInfo {
						field: Some(field),
						command,
					};
					let commands = [commands_with_field];

					match target {
						Some(target_user) => {
							self.send_to_user(&target_user, template, &commands);
						}
						None => {
							self.send_to_users(groups, template, &commands, |user| {
								let mut permission_manager = permission_manager.borrow_mut();
								// отправляем себе только если есть права на запись
								// иначе никто другой не может вносит изменения в данное поле и
								// отправлять себе как единственному источнику изменений избыточно
								if object_owner == Option::Some(user.id) {
									permission_manager.has_write_access(template, field)
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
