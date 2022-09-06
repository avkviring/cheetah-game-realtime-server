use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
use cheetah_matches_realtime_common::room::RoomMemberId;

use crate::room::command::ServerCommandError;
use crate::room::object::{Field, GameObject, S2CCommandWithFieldInfo};
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
		game_object_id: &GameObjectId,
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
		let permission_manager = self.permission_manager.clone();
		let creator_access_group = match self.members.get(&creator_id) {
			None => {
				return Result::Err(ServerCommandError::MemberNotFound(creator_id));
			}
			Some(member) => member.template.groups,
		};

		let object = self.get_object(game_object_id)?;
		// проверяем группу доступа
		if !object.access_groups.contains_any(&creator_access_group) {
			return Result::Err(ServerCommandError::MemberCannotAccessToObject {
				room_id,
				member_id: creator_id,
				object_id: game_object_id.clone(),
				member_access_group: creator_access_group,
				object_access_group: object.access_groups,
			});
		}

		let object_owner = if let GameObjectOwner::Member(owner) = object.id.owner {
			Option::Some(owner)
		} else {
			Option::None
		};

		let is_creator_object_owner = object_owner == Option::Some(creator_id);

		let allow = is_creator_object_owner
			|| permission_manager
				.borrow_mut()
				.get_permission(object.template_id, field, creator_access_group)
				>= permission;

		if !allow {
			return Result::Err(ServerCommandError::MemberCannotAccessToObjectField {
				room_id,
				member_id: creator_id,
				object_id: object.id.clone(),
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

				let commands_with_field = S2CCommandWithFieldInfo { field: Some(field), command };
				let commands = [commands_with_field];

				match target {
					Some(target_user) => {
						self.send_to_member(&target_user, template, &commands)?;
					}
					None => {
						self.send_to_members(groups, template, &commands, |user| {
							let mut permission_manager = permission_manager.borrow_mut();
							// отправляем себе только если есть права на запись
							// иначе никто другой не может вносит изменения в данное поле и
							// отправлять себе как единственному источнику изменений избыточно
							if object_owner == Option::Some(user.id) {
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
