use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use cheetah_relay::server::Server;
use cheetah_relay::test_env::IntegrationTestServerBuider;
use cheetah_relay_client::ffi::client::do_create_client;
use cheetah_relay_client::registry::ClientId;

use cheetah_relay_common::room::{UserId, UserPrivateKey};

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	pub server: Server,
}

impl IntegrationTestHelper {
	pub fn new(builder: IntegrationTestServerBuider) -> Self {
		let (socket_addr, server) = builder.build();
		Self { socket_addr, server }
	}

	pub fn create_client(&self, user_id: UserId, user_key: UserPrivateKey) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(
			self.socket_addr.to_string(),
			user_id,
			IntegrationTestServerBuider::ROOM_ID,
			&user_key,
			0,
			&mut client,
		);
		client
	}

	pub fn wait_first_frame(&self) {
		thread::sleep(Duration::from_millis(100));
	}
}

pub fn setup() -> (IntegrationTestHelper, u16, u16) {
	let mut builder = IntegrationTestServerBuider::default();
	let (user1_id, user1_key) = builder.create_user();
	let (user2_id, user2_key) = builder.create_user();
	let helper = IntegrationTestHelper::new(builder);

	let client1 = helper.create_client(user1_id, user1_key);
	let client2 = helper.create_client(user2_id, user2_key);
	(helper, client1, client2)
}
