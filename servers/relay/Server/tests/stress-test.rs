use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use log::LevelFilter;

use cheetah_relay::room::template::{GameObjectTemplate, RoomTemplate, UserTemplate};
use cheetah_relay::server::Server;
use cheetah_relay_common::commands::command::long::SetLongCommand;
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::udp::bind_to_free_socket;
use cheetah_relay_common::udp::client::UdpClient;

///
/// Тестируем работу сервера под большой нагрузкой
///
#[test]
pub fn stress_test() {
	let socket = bind_to_free_socket().unwrap();
	let addr = socket.1;
	let mut server = Server::new(socket.0);
	let object_template = GameObjectTemplate {
		id: 0,
		template: 0,
		access_groups: AccessGroups(0b1),
		fields: Default::default(),
	};
	let user1 = UserTemplate {
		public_key: 1,
		private_key: [0; 32],
		access_groups: AccessGroups(0b1),
		objects: Option::Some(vec![object_template.clone()]),
	};

	let user2 = UserTemplate {
		public_key: 2,
		private_key: [0; 32],
		access_groups: AccessGroups(0b1),
		objects: None,
	};

	server
		.register_room(RoomTemplate {
			id: 0,
			auto_create_user: false,
			users: vec![user1.clone(), user2.clone()],
			objects: None,
		})
		.ok()
		.unwrap();

	let mut client1 = UdpClient::new(user1.private_key.clone(), user1.public_key.clone(), addr.clone(), 100).unwrap();
	let mut client2 = UdpClient::new(user2.private_key.clone(), user2.public_key.clone(), addr.clone(), 100).unwrap();

	client2.protocol.out_commands_collector.add_command(
		ApplicationCommandChannelType::ReliableSequenceByGroup(0),
		ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
			meta: C2SMetaCommandInformation { timestamp: 0 },
			command: C2SCommand::AttachToRoom,
		}),
	);
	cycle(&mut client1, &mut client2);
	thread::sleep(Duration::from_millis(10));

	let count = 2000;
	for i in 0..count {
		let command = SetLongCommand {
			object_id: GameObjectId {
				owner: ObjectOwner::User(user1.public_key),
				id: object_template.id,
			},
			field_id: 1,
			value: i,
		};
		client1.protocol.out_commands_collector.add_command(
			ApplicationCommandChannelType::ReliableSequenceByGroup(0),
			ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
				meta: C2SMetaCommandInformation { timestamp: 0 },
				command: C2SCommand::SetLongValue(command),
			}),
		);
		cycle(&mut client1, &mut client2);
	}

	thread::sleep(Duration::from_millis(10));
	cycle(&mut client1, &mut client2);

	let in_commands = client2.protocol.in_commands_collector.get_commands();
	assert_eq!(in_commands.len(), count as usize + 1); // +1 - команда создания объекта
}

fn cycle(client1: &mut UdpClient, client2: &mut UdpClient) {
	let now = Instant::now();
	client1.cycle(&now);
	client2.cycle(&now);
	thread::sleep(Duration::from_millis(1));
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
