use std::ops::Add;
use std::time::{Duration, Instant};

use cheetah_relay_common::protocol::channel::Transport;
use cheetah_relay_common::protocol::client::UdpClient;
use cheetah_relay_common::protocol::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel};
use cheetah_relay_common::protocol::server::UdpServer;

use crate::udp::stub::{AddressStub, ChannelQuality, create_user_private_key_stub, create_user_public_key_stub, TransportStub};

///
/// Тестирование отправки команд с клиента на сервер
///
#[test]
fn should_send_from_client() {
	let transport = TransportStub::new(ChannelQuality::default());
	let (mut server, public_key, mut client) = setup(transport);


	client.protocol.out_commands_collector.add_command(ApplicationCommandChannel::ReliableUnordered, ApplicationCommand::TestSimple("test reliability".to_string()));
	client.protocol.out_commands_collector.add_command(ApplicationCommandChannel::UnreliableUnordered, ApplicationCommand::TestSimple("test unreliability".to_string()));
	
	let now = Instant::now();
	client.cycle(&now);
	server.cycle(&now);
	
	let protocol = &mut server.get_user_sessions(&public_key).protocol;
	let commands = protocol.in_commands_collector.get_commands();
	
	assert!(commands.iter().find(|p| matches!(&p, ApplicationCommand::TestSimple(v) if *v == "test reliability".to_string())).is_some());
	assert!(commands.iter().find(|p| matches!(&p, ApplicationCommand::TestSimple(v) if *v == "test unreliability".to_string())).is_some());
}

///
/// Тестирование отправки команды с сервера на клиент
///
#[test]
fn should_send_from_server() {
	let transport = TransportStub::new(ChannelQuality::default());
	let (mut server, public_key, mut client) = setup(transport);
	
	let now = Instant::now();
	client.cycle(&now);
	server.cycle(&now);
	
	let protocol = &mut server.get_user_sessions(&public_key).protocol;
	let ping_message = "ping from server".to_string();
	protocol.out_commands_collector.add_command(ApplicationCommandChannel::ReliableUnordered, ApplicationCommand::TestSimple(ping_message));

	server.cycle(&now);
	client.cycle(&now);
	
	let commands = client.protocol.in_commands_collector.get_commands();
	//ApplicationCommand::Ping()
	assert!(commands.iter().find(|p| matches!(&p, ApplicationCommand::TestSimple(ping_message))).is_some());
}

///
/// Тестирование надежной доставки по ненадежному каналу
#[test]
fn should_transfer_reliable_on_unreliable_channel() {
	let mut channel_quality = ChannelQuality::default();
	channel_quality.add_reliable_percent(0..=5, 0.0);
	let transport = TransportStub::new(channel_quality);
	
	let (mut server, public_key, mut client) = setup(transport);
	client.protocol.out_commands_collector.add_command(ApplicationCommandChannel::ReliableUnordered, ApplicationCommand::TestSimple("test".to_string()));

	
	let mut now = Instant::now();
	for i in 0..6 {
		now = now.add(Duration::from_millis(300));
		client.cycle(&now);
		server.cycle(&now);
	}
	
	let protocol = &mut server.get_user_sessions(&public_key).protocol;
	let commands = protocol.in_commands_collector.get_commands();
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::TestSimple(_))).is_some());
}


fn setup(transport: Box<dyn Transport<AddressStub>>) -> (UdpServer<AddressStub>, [u8; 4], UdpClient<AddressStub>) {
	let server_address = AddressStub::new(0);
	let channel = transport.create_channel(server_address.clone()).ok().unwrap();
	let mut server = UdpServer::new(channel);
	
	let private_key = create_user_private_key_stub();
	let public_key = create_user_public_key_stub();
	server.add_allowed_user(private_key, public_key);
	
	let client_address = AddressStub::new(1);
	let client_channel = transport.create_channel(client_address).ok().unwrap();
	let mut client = UdpClient::new(private_key, public_key, client_channel, server_address);
	(server, public_key, client)
}


