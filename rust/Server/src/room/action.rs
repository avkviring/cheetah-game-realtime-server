use cheetah_common::commands::field::Field;
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomMemberId;
use std::rc::Rc;

use crate::room::command::ServerCommandError;
use crate::room::object::GameObject;
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
	pub fn send_command_from_action<T>(
		&mut self,
		game_object_id: GameObjectId,
		field: Field,
		creator_id: RoomMemberId,
		permission: Permission,
		target: Option<RoomMemberId>,
		action: T,
	) -> Result<(), ServerCommandError>
	where
		T: FnOnce(&mut GameObject) -> Result<Option<S2CCommand>, ServerCommandError>,
	{
		let room_id = self.id;
		let permission_manager = Rc::clone(&self.permission_manager);
		let creator_access_group = match self.members.get(&creator_id) {
			None => {
				return Err(ServerCommandError::MemberNotFound(creator_id));
			}
			Some(member) => member.template.groups,
		};

		let object = self.get_object_mut(game_object_id)?;
		// проверяем группу доступа
		if !object.access_groups.contains_any(&creator_access_group) {
			return Err(ServerCommandError::MemberCannotAccessToObject {
				room_id,
				member_id: creator_id,
				object_id: game_object_id,
				member_access_group: creator_access_group,
				object_access_group: object.access_groups,
			});
		}

		let object_owner = if let GameObjectOwner::Member(owner) = object.id.get_owner() { Some(owner) } else { None };

		let is_creator_object_owner = object_owner == Some(creator_id);

		let allow = is_creator_object_owner || permission_manager.borrow_mut().get_permission(object.template_id, field, creator_access_group) >= permission;

		if !allow {
			return Err(ServerCommandError::MemberCannotAccessToObjectField {
				room_id,
				member_id: creator_id,
				object_id: object.id,
				template_id: object.template_id,
				field,
			});
		}

		let command = action(object)?;
		if let Some(command) = command {
			// отправляем команду только для созданного объекта
			if object.created {
				let groups = object.access_groups;
				let template = object.template_id;

				let commands_with_field = S2CCommandWithMeta {
					field: Some(field),
					creator: creator_id,
					command,
				};
				let commands = [commands_with_field];

				match target {
					Some(target_member_id) => {
						self.send_to_member(&target_member_id, template, &commands)?;
					}
					None => {
						self.send_to_members(groups, Some(template), &commands, |member| {
							let permission_manager = permission_manager.borrow_mut();
							// отправляем себе только если есть права на запись
							// иначе никто другой не может вносит изменения в данное поле и
							// отправлять себе как единственному источнику изменений избыточно
							if object_owner == Some(member.id) {
								permission_manager.has_write_access(template, field)
							} else {
								true
							}
						})?;
					}
				}
			}
		}
		Ok(())
	}
}
