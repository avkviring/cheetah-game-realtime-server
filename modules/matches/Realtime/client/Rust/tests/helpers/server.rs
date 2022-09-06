use std::net::SocketAddr;

use cheetah_matches_realtime::room::object::Field;
use cheetah_matches_realtime::room::template::config::{
	GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField, RoomTemplate,
};
use cheetah_matches_realtime::server::manager::ServerManager;
use cheetah_matches_realtime_common::commands::FieldType;
use cheetah_matches_realtime_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_realtime_common::network::bind_to_free_socket;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::RoomId;

///
/// Конфигурируем и создаем сервер для интеграционного тестирования
///
#[derive(Debug, Default)]
pub struct IntegrationTestServerBuilder {
	template: RoomTemplate,
}

impl IntegrationTestServerBuilder {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(55);
	pub const DEFAULT_TEMPLATE: GameObjectTemplateId = 1;

	pub fn set_permission(
		&mut self,
		template: GameObjectTemplateId,
		field_id: FieldId,
		field_type: FieldType,
		group: AccessGroups,
		permission: Permission,
	) {
		let field = PermissionField {
			field: Field { id: field_id, field_type },
			rules: vec![GroupsPermissionRule { groups: group, permission }],
		};
		match self.template.permissions.templates.iter_mut().find(|tp| tp.template == template) {
			None => self.template.permissions.templates.push(GameObjectTemplatePermission {
				template,
				rules: Default::default(),
				fields: vec![field],
			}),
			Some(template) => {
				template.fields.push(field);
			}
		}
	}

	pub fn build(self) -> (SocketAddr, ServerManager, RoomId) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.1;
		let mut server = ServerManager::new(socket.0);
		let room_id = server.register_room(self.template).ok().unwrap();
		(addr, server, room_id)
	}
}
