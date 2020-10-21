use std::time::Duration;

use cheetah_relay_common::udp::channel::stub::{AddressStub, TransportStub};
use cheetah_relay_common::udp::client::UdpClient;
use cheetah_relay_common::udp::protocol::format::applications::ApplicationCommand;
use cheetah_relay_common::udp::server::UdpServer;

use crate::udp::{create_user_private_key_stub, create_user_public_key_stub};

#[test]
fn should_send_and_receive_commands() {
	let transport = TransportStub::new();
	let server_address = AddressStub::new(0);
	let mut server = UdpServer::new(transport.create_channel(server_address.clone()));
	
	let private_key = create_user_private_key_stub();
	let public_key = create_user_public_key_stub();
	server.add_allowed_user(private_key, public_key);
	
	let client_address = AddressStub::new(1);
	let mut client = UdpClient::new(private_key, public_key, transport.create_channel(client_address), server_address);
	
	client.protocol.out_commands_collector.add_reliability_command(ApplicationCommand::Ping("test reliability".to_string()));
	client.protocol.out_commands_collector.add_unreliability_command(ApplicationCommand::Ping("test unreliability".to_string()));
	
	client.cycle(&Duration::new(0, 0));
	server.cycle(&Duration::new(0, 0));
	
	let mut protocol = &mut server.get_user_sessions(&public_key).protocol;
	let commands = protocol.in_commands_collector.get_and_remove_commands();
	
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::Ping(v) if *v == "test reliability".to_string())).is_some());
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::Ping(v) if *v == "test unreliability".to_string())).is_some());
}