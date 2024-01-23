use cheetah_common::network::bind_to_free_socket;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectTemplateId;
use cheetah_game_realtime_protocol::coniguration::ProtocolConfiguration;
use cheetah_game_realtime_protocol::RoomId;
use cheetah_server::server::manager::ServerManager;
use cheetah_server::server::room::config::room::RoomCreateParams;
use std::net::SocketAddr;
use std::time::Duration;

///
/// Конфигурируем и создаем сервер для интеграционного тестирования
///
#[derive(Debug, Default)]
pub struct IntegrationTestServerBuilder {
	template: RoomCreateParams,
}

impl IntegrationTestServerBuilder {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(55);
	pub const DEFAULT_TEMPLATE: GameObjectTemplateId = 1;
	pub const DISCONNECT_DURATION: Duration = Duration::from_secs(30);

	#[must_use]
	pub fn build(self) -> (SocketAddr, ServerManager, RoomId) {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.local_addr().unwrap();
		let mut server = ServerManager::new(
			socket,
			ProtocolConfiguration {
				disconnect_timeout: Self::DISCONNECT_DURATION,
			},
		)
		.unwrap();
		let room_id = server.create_room(self.template).ok().unwrap();
		(addr, server, room_id)
	}
}
