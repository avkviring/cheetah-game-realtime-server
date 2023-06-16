use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use cheetah_client::clients::registry::ClientId;
use cheetah_client::ffi;
use cheetah_client::ffi::client::do_create_client;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::object::GameObjectId;
use cheetah_protocol::frame::member_private_key::MemberPrivateKey;
use cheetah_protocol::{RoomId, RoomMemberId};
use cheetah_server::room::template::config::MemberTemplate;
use cheetah_server::server::manager::ServerManager;

use crate::helpers::server::IntegrationTestServerBuilder;

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	pub room_id: RoomId,
	pub server: ServerManager,
}

impl IntegrationTestHelper {
	pub fn receive(&self, client: ClientId) -> Vec<S2CCommandFFI> {
		let mut commands = vec![
			S2CCommandFFI {
				command_type: CommandTypeId::CreateGameObject,
				command: S2CommandUnionFFI { empty: () },
			};
			1024
		];
		self.wait_udp();

		let mut count = 0;
		ffi::client::receive(client, commands.as_mut_ptr(), &mut count);
		commands[0..count as usize].to_vec()
	}
}

impl IntegrationTestHelper {
	#[must_use]
	pub fn new(builder: IntegrationTestServerBuilder) -> Self {
		let (socket_addr, server, room_id) = builder.build();
		Self { socket_addr, room_id, server }
	}

	pub fn create_client(&self, member_id: RoomMemberId, private_key: &MemberPrivateKey) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(0, &self.socket_addr.to_string(), member_id, self.room_id, private_key, &mut client);
		client
	}

	pub fn wait_udp(&self) {
		thread::sleep(Duration::from_millis(200));
	}

	pub fn create_member_object(&self, client_id: ClientId) -> GameObjectId {
		let mut object_id = GameObjectId::default();
		ffi::command::object::create_object(
			client_id,
			IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
			IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0,
			&mut object_id,
		);
		ffi::command::object::created_object(client_id, &object_id, false, &Default::default());
		object_id
	}

	pub fn create_member(&mut self) -> (RoomMemberId, MemberPrivateKey) {
		let private_key = MemberPrivateKey::new_random();
		let member_template = MemberTemplate {
			super_member: false,
			private_key: private_key.clone(),
			groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
		};
		let member_id = self.server.create_member(self.room_id, member_template).ok().unwrap();
		(member_id, private_key)
	}
}

#[must_use]
pub fn setup<const N: usize>(builder: IntegrationTestServerBuilder) -> (IntegrationTestHelper, [u16; N]) {
	let mut helper = IntegrationTestHelper::new(builder);
	let mut members = [0; N];
	for member in members.iter_mut() {
		let (member_id, private_key) = helper.create_member();
		*member = helper.create_client(member_id, &private_key);
	}
	(helper, members)
}
