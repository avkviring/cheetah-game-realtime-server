use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use cheetah_matches_realtime::room::template::config::MemberTemplate;
use cheetah_matches_realtime::server::manager::ServerManager;
use cheetah_matches_realtime_client::clients::registry::ClientId;
use cheetah_matches_realtime_client::ffi;
use cheetah_matches_realtime_client::ffi::client::do_create_client;
use cheetah_matches_realtime_client::ffi::{BufferFFI, GameObjectIdFFI};
use cheetah_matches_realtime_common::room::{MemberPrivateKey, RoomId, RoomMemberId};

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

	pub fn create_client(&self, user_id: RoomMemberId, user_key: MemberPrivateKey) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(
			self.socket_addr.to_string(),
			user_id,
			self.room_id,
			&user_key,
			0,
			&mut client,
		);
		client
	}

	pub fn wait_udp(&self) {
		thread::sleep(Duration::from_millis(1000));
	}

	pub fn create_member_object(&self, client_id: ClientId) -> GameObjectIdFFI {
		let mut object_id = GameObjectIdFFI::default();
		ffi::command::object::create_object(
			client_id,
			IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
			IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
			&mut object_id,
		);
		ffi::command::object::created_object(client_id, &object_id, false, &BufferFFI::default());
		object_id
	}

	pub fn create_user(&mut self) -> (RoomMemberId, MemberPrivateKey) {
		let private_key = MemberPrivateKey::new_random();
		let user_template = MemberTemplate {
			super_member: false,
			private_key: private_key.clone(),
			groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
		};
		let user_id = self
			.server
			.register_user(self.room_id, user_template)
			.ok()
			.unwrap();
		(user_id, private_key)
	}
}

pub fn setup<const N: usize>(
	builder: IntegrationTestServerBuilder,
) -> (IntegrationTestHelper, [u16; N]) {
	let mut helper = IntegrationTestHelper::new(builder);
	let mut members = [0; N];
	for i in 0..N {
		let (user_id, user_key) = helper.create_user();
		let client = helper.create_client(user_id, user_key);
		members[i] = client;
	}
	(helper, members)
}
