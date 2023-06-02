use std::net::SocketAddr;

use fnv::FnvHashSet;

use cheetah_common::network::bind_to_free_socket;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::field::{Field, FieldId, FieldType};
use cheetah_common::room::object::GameObjectTemplateId;
use cheetah_protocol::RoomId;
use cheetah_server::room::template::config::{GameObjectTemplatePermission, GroupsPermissionRule, Permission, PermissionField, RoomTemplate};
use cheetah_server::server::manager::RoomsServerManager;

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

	pub fn set_permission(&mut self, template: GameObjectTemplateId, field_id: FieldId, field_type: FieldType, group: AccessGroups, permission: Permission) {
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

	#[must_use]
	pub fn build(self) -> (SocketAddr, RoomsServerManager, RoomId) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.local_addr().unwrap();
		let mut server = RoomsServerManager::new(socket, FnvHashSet::default()).unwrap();
		let room_id = server.create_room(self.template).ok().unwrap();
		(addr, server, room_id)
	}
}
