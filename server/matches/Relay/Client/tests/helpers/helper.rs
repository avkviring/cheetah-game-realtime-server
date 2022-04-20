use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use rand::rngs::OsRng;
use rand::RngCore;

use cheetah_matches_relay::room::template::config::MemberTemplate;
use cheetah_matches_relay::server::manager::ServerManager;
use cheetah_matches_relay_client::clients::registry::ClientId;
use cheetah_matches_relay_client::ffi;
use cheetah_matches_relay_client::ffi::client::do_create_client;
use cheetah_matches_relay_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::helpers::server::IntegrationTestServerBuilder;

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	room_id: RoomId,
	pub server: ServerManager,
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
		thread::sleep(Duration::from_millis(1000));
	}

	pub fn create_member_object(&self, client_id: ClientId) -> GameObjectIdFFI {
		let mut object_id = GameObjectIdFFI::default();
		ffi::command::object::create_member_object(
			client_id,
			IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
			IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
			&mut object_id,
		);
		ffi::command::object::created_object(client_id, &object_id, false, false, &BufferFFI::default());
		object_id
	}

	pub fn create_user(&mut self) -> (RoomMemberId, UserPrivateKey) {
		let mut private_key = [0; 32];
		OsRng.fill_bytes(&mut private_key);
		let user_template = MemberTemplate {
			private_key,
			groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
		};
		let user_id = self.server.register_user(self.room_id, user_template).ok().unwrap();
		(user_id, private_key)
	}
}

pub fn setup<const N: usize>(builder: IntegrationTestServerBuilder) -> (IntegrationTestHelper, [u16; N]) {
	let mut helper = IntegrationTestHelper::new(builder);
	let mut members = [0; N];
	for i in 0..N {
		let (user_id, user_key) = helper.create_user();
		let client = helper.create_client(user_id, user_key);
		members[i] = client;
	}
	(helper, members)
}
