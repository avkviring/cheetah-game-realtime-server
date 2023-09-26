use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::room::field::Field;
use cheetah_common::room::object::GameObjectId;
use cheetah_game_realtime_protocol::RoomMemberId;
use crate::server::room::command::ServerCommandError;
use crate::server::room::object::GameObject;
use crate::server::room::Room;

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
	pub fn send_command_from_action<T>(&mut self, game_object_id: GameObjectId, field: Field, creator_id: RoomMemberId, target: Option<RoomMemberId>, action: T) -> Result<(), ServerCommandError>
	where
		T: FnOnce(&mut GameObject) -> Result<Option<S2CCommand>, ServerCommandError>,
	{
		let room_id = self.id;
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

		let command = action(object)?;
		if let Some(command) = command {
			// отправляем команду только для созданного объекта
			if object.created {
				let groups = object.access_groups;
				let commands_with_field = S2CCommandWithMeta {
					field: Some(field),
					creator: creator_id,
					command,
				};
				let commands = [commands_with_field];

				match target {
					Some(target_member_id) => {
						self.send_to_member(&target_member_id, &commands)?;
					}
					None => {
						self.send_to_members(groups, &commands, |member| {
							// не отправляем себе
							creator_id != member.id
						})?;
					}
				}
			}
		}
		Ok(())
	}
}
