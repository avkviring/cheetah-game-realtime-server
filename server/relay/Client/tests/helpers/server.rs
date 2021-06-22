use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;

use log::LevelFilter;
use rand::rngs::OsRng;
use rand::RngCore;

use cheetah_relay::room::debug::tracer::CommandTracer;
use cheetah_relay::room::template::config::{
	GameObjectTemplate, Permission, PermissionField, PermissionGroup, RoomTemplate, TemplatePermission, UserTemplate,
};
use cheetah_relay::room::types::FieldType;
use cheetah_relay::server::RelayServer;
use cheetah_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_relay_common::network::bind_to_free_socket;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::{RoomId, UserId, UserPrivateKey};

///
/// Конфигурируем и создаем сервер для интеграционного тестирования
///
#[derive(Debug, Default)]
pub struct IntegrationTestServerBuilder {
	user_id_generator: UserId,
	object_id_generator: u32,
	template: RoomTemplate,
	users: HashMap<UserId, UserTemplate>,
	enable_trace: bool,
}

impl IntegrationTestServerBuilder {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(55);
	pub const DEFAULT_TEMPLATE: GameObjectTemplateId = 1;

	pub fn create_user(&mut self) -> (UserId, UserPrivateKey) {
		self.user_id_generator += 1;
		let mut private_key = [0; 32];
		OsRng.fill_bytes(&mut private_key);
		self.users.insert(
			self.user_id_generator,
			UserTemplate {
				id: self.user_id_generator,
				private_key,
				access_groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
				objects: Default::default(),
			},
		);
		(self.user_id_generator, private_key)
	}

	pub fn create_object(&mut self, user_id: UserId, template: GameObjectTemplateId) -> GameObjectId {
		self.object_id_generator += 1;
		let object_template = GameObjectTemplate {
			id: self.object_id_generator,
			template,
			access_groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
			fields: Default::default(),
		};

		let user = self.users.get_mut(&user_id).unwrap();
		user.objects.push(object_template);
		GameObjectId {
			owner: ObjectOwner::User(user_id),
			id: self.object_id_generator,
		}
	}

	pub fn set_permission(
		&mut self,
		template: GameObjectTemplateId,
		field_id: FieldId,
		field_type: FieldType,
		group: AccessGroups,
		permission: Permission,
	) {
		let field = PermissionField {
			field_id,
			field_type,
			groups: vec![PermissionGroup { group, permission }],
		};
		match self.template.permissions.templates.iter_mut().find(|tp| tp.template == template) {
			None => self.template.permissions.templates.push(TemplatePermission {
				template,
				groups: Default::default(),
				fields: vec![field],
			}),
			Some(template) => {
				template.fields.push(field);
			}
		}
	}

	pub fn enable_trace(&mut self) {
		self.enable_trace = true;
	}

	pub fn build(self) -> (SocketAddr, RelayServer, RoomId) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.1;
		let tracer = if self.enable_trace {
			init_logger();
			CommandTracer::new_with_allow_all()
		} else {
			CommandTracer::new_with_deny_all()
		};

		let mut server = RelayServer::new(socket.0, tracer);
		let room_id = server.register_room(self.template).ok().unwrap();
		for (_, user) in self.users {
			server.register_user(room_id, user).ok().unwrap();
		}
		(addr, server, room_id)
	}
}

#[allow(unused)]
fn init_logger() {
	env_logger::builder()
		.format(|buf, record| {
			writeln!(
				buf,
				"[{}] [{}] {:?}",
				record.level(),
				std::thread::current().name().unwrap_or(""),
				record.args()
			)
		})
		.filter_level(LevelFilter::Info)
		.format_timestamp(None)
		.is_test(true)
		.try_init();
}
