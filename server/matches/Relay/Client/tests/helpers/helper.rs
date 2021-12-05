use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use rand::rngs::OsRng;
use rand::RngCore;

use cheetah_matches_relay::room::template::config::UserTemplate;
use cheetah_matches_relay::server::manager::RelayManager;
use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::client::do_create_client;
use cheetah_matches_relay_client::ffi::GameObjectIdFFI;
use cheetah_matches_relay_client::registry::ClientId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::helpers::server::IntegrationTestServerBuilder;

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	room_id: RoomId,
	pub server: RelayManager,
}

impl IntegrationTestHelper {
	pub fn new(builder: IntegrationTestServerBuilder) -> Self {
		let (socket_addr, server, room_id) = builder.build();
		Self {
			socket_addr,
			room_id,
			server,
		}
	}

	pub fn create_client(&self, user_id: RoomMemberId, user_key: UserPrivateKey) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(self.socket_addr.to_string(), user_id, self.room_id, &user_key, 0, &mut client);
		client
	}

	pub fn wait_udp(&self) {
		thread::sleep(Duration::from_millis(500));
	}

	pub fn create_user_object(&self, client_id: ClientId) -> GameObjectIdFFI {
		let mut object_id = GameObjectIdFFI::new();
		ffi::command::object::create_object(
			client_id,
			IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
			IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
			&mut object_id,
		);
		ffi::command::object::created_object(client_id, &object_id);
		object_id
	}

	pub fn create_user(&mut self) -> (RoomMemberId, UserPrivateKey) {
		let mut private_key = [0; 32];
		OsRng.fill_bytes(&mut private_key);
		let user_template = UserTemplate {
			private_key,
			groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
		};
		let user_id = self.server.register_user(self.room_id, user_template).ok().unwrap();
		(user_id, private_key)
	}
}

pub fn setup(builder: IntegrationTestServerBuilder) -> (IntegrationTestHelper, u16, u16) {
	let mut helper = IntegrationTestHelper::new(builder);
	let (user1_id, user1_key) = helper.create_user();
	let (user2_id, user2_key) = helper.create_user();

	let client1 = helper.create_client(user1_id, user1_key);
	let client2 = helper.create_client(user2_id, user2_key);
	(helper, client1, client2)
}
