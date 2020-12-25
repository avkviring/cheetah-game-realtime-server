use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};

use log::LevelFilter;

use cheetah_relay::room::debug::tracer::CommandTracer;
use cheetah_relay::room::template::config::{GameObjectTemplate, RoomTemplate, UserTemplate};
use cheetah_relay::server::Server;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::{RoomId, UserId};
use cheetah_relay_common::udp::bind_to_free_socket;
use cheetah_relay_common::udp::client::UdpClient;

pub struct TestEnv {
	socket_addr: SocketAddr,
	clients: HashMap<UserId, UdpClient>,
	pub server: Server,
}

impl TestEnv {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(0b1);

	pub fn connect(&mut self, user_id: UserId, room_id: RoomId) {
		let client = UdpClient::new(Default::default(), user_id, room_id, self.socket_addr, 100).unwrap();
		self.clients.insert(user_id, client);
	}

	pub fn cycle(&mut self) {
		for _ in 0..2 {
			let now = Instant::now();
			self.clients.iter_mut().for_each(|(_, client)| {
				client.cycle(&now);
			});
			thread::sleep(Duration::from_millis(5));
		}
	}

	pub fn send_to_server(&mut self, user_id: UserId, command: C2SCommand) {
		let client = self.clients.get_mut(&user_id).unwrap();
		client.protocol.out_commands_collector.add_command(
			ApplicationCommandChannelType::ReliableSequenceByGroup(0),
			ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
				meta: C2SMetaCommandInformation { timestamp: 0 },
				command,
			}),
		)
	}
	pub fn get_input_commands(&mut self, user_id: UserId) -> Vec<S2CCommand> {
		let client = self.clients.get_mut(&user_id).unwrap();
		let commands = client.protocol.in_commands_collector.get_commands();
		let result = commands
			.into_iter()
			.rev()
			.map(|c| c.command.clone())
			.map(|c| match c {
				ApplicationCommand::S2CCommandWithMeta(command) => Option::Some(command.command),
				_ => Option::None,
			})
			.filter(|o| o.is_some())
			.map(|o| o.unwrap())
			.collect();
		commands.clear();
		result
	}
}

#[derive(Debug, Default)]
pub struct TestEnvBuilder {
	template: RoomTemplate,
}

impl TestEnvBuilder {
	pub const ROOM_ID: RoomId = 0;

	pub fn create_user(&mut self, user_id: UserId) {
		self.template.users.push(UserTemplate {
			id: user_id,
			private_key: Default::default(),
			access_groups: TestEnv::DEFAULT_ACCESS_GROUP,
			objects: Default::default(),
			unmapping: Default::default(),
		});
	}

	pub fn create_object(&mut self, user_id: UserId, object_id: u32) {
		let object_template = GameObjectTemplate {
			id: object_id,
			template: 0,
			access_groups: TestEnv::DEFAULT_ACCESS_GROUP,
			fields: Default::default(),
			unmapping: Default::default(),
		};

		let user = self.template.users.iter_mut().find(|u| u.id == user_id).unwrap();
		user.objects.push(object_template);
	}

	pub fn build(self) -> TestEnv {
		let socket = bind_to_free_socket().unwrap();
		let addr = socket.1;
		let mut server = Server::new(socket.0, CommandTracer::new_with_deny_all());
		server.register_room(self.template).ok().unwrap();
		TestEnv {
			socket_addr: addr,
			clients: Default::default(),
			server,
		}
	}
}

#[allow(dead_code)]
pub fn init_logger() {
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
		.init();
}
