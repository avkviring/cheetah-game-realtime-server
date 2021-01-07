use std::net::SocketAddr;

use rand::rngs::OsRng;
use rand::RngCore;

use cheetah_relay_common::constants::GameObjectTemplateId;
use cheetah_relay_common::network::bind_to_free_socket;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::{RoomId, UserId, UserPrivateKey};

use crate::room::debug::tracer::CommandTracer;
use crate::room::template::config::{GameObjectTemplate, RoomTemplate, UserTemplate};
use crate::server::Server;

///
/// Конфигурируем и создаем сервер для интеграционного тестирования
///
#[derive(Debug, Default)]
pub struct IntegrationTestServerBuider {
	user_id_generator: UserId,
	object_id_generator: u32,
	template: RoomTemplate,
}

impl IntegrationTestServerBuider {
	pub const ROOM_ID: RoomId = 0;

	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(55);

	pub fn create_user(&mut self) -> (UserId, UserPrivateKey) {
		self.user_id_generator += 1;
		let mut private_key = [0; 32];
		OsRng.fill_bytes(&mut private_key);
		self.template.users.push(UserTemplate {
			id: self.user_id_generator,
			private_key: private_key.clone(),
			access_groups: IntegrationTestServerBuider::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
			unmapping: Default::default(),
		});
		(self.user_id_generator, private_key)
	}

	pub fn create_object(&mut self, user_id: UserId, template: GameObjectTemplateId) -> GameObjectId {
		self.object_id_generator += 1;
		let object_template = GameObjectTemplate {
			id: self.object_id_generator,
			template,
			access_groups: IntegrationTestServerBuider::DEFAULT_ACCESS_GROUP,
			fields: Default::default(),
			unmapping: Default::default(),
		};

		let user = self.template.users.iter_mut().find(|u| u.id == user_id).unwrap();
		user.objects.push(object_template);
		GameObjectId {
			owner: ObjectOwner::User(user_id),
			id: self.object_id_generator,
		}
	}

	pub fn build(self) -> (SocketAddr, Server) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.1;
		let mut server = Server::new(socket.0, CommandTracer::new_with_deny_all());
		server.register_room(self.template).ok().unwrap();
		(addr, server)
	}
}
