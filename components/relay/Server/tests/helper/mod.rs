use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};

use log::LevelFilter;

use cheetah_relay::server::Server;
use cheetah_relay::test_env::IntegrationTestServerBuider;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};

use cheetah_relay_common::network::client::NetworkClient;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::{UserId, UserPrivateKey};

pub struct IntegrationTestHelper {
	socket_addr: SocketAddr,
	clients: HashMap<UserId, NetworkClient>,
	pub server: Server,
}

impl IntegrationTestHelper {
	pub const DEFAULT_ACCESS_GROUP: AccessGroups = AccessGroups(0b1);

	pub fn new(builder: IntegrationTestServerBuider) -> Self {
		let (socket_addr, server) = builder.build();
		Self {
			socket_addr,
			clients: Default::default(),
			server,
		}
	}

	pub fn connect(&mut self, user_id: UserId, user_key: UserPrivateKey) {
		let client = NetworkClient::new(user_key, user_id, IntegrationTestServerBuider::ROOM_ID, self.socket_addr, 100).unwrap();
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
				meta: C2SMetaCommandInformation::default(),
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
