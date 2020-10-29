use rand::rngs::OsRng;

use cheetah_relay_common::commands::hash::{UserPrivateKey, UserPublicKey};
use rand::RngCore;
use cheetah_relay_common::udp::protocol::frame::applications::ApplicationCommand;
use std::time::Instant;
use cheetah_relay_common::udp::channel::Transport;
use cheetah_relay_common::udp::server::UdpServer;
use cheetah_relay_common::udp::client::UdpClient;
use crate::udp::stub::{TransportStub, AddressStub};

pub mod stub;


#[test]
fn should_send_and_receive_commands() {
	let transport = TransportStub::new();
	
	let (mut server, public_key, mut client, mut transport) = setup(transport);
	
	client.protocol.out_commands_collector.add_reliability_command(ApplicationCommand::Ping("test reliability".to_string()));
	client.protocol.out_commands_collector.add_unreliability_command(ApplicationCommand::Ping("test unreliability".to_string()));
	
	let now = Instant::now();
	client.cycle(&now);
	server.cycle(&now);
	
	let protocol = &mut server.get_user_sessions(&public_key).protocol;
	let commands = protocol.in_commands_collector.get_and_remove_commands();
	
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::Ping(v) if *v == "test reliability".to_string())).is_some());
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::Ping(v) if *v == "test unreliability".to_string())).is_some());
}


fn setup(transport: Box<Transport<AddressStub>>) -> (UdpServer<AddressStub>, [u8; 4], UdpClient<AddressStub>, Box<dyn Transport<AddressStub>>) {
	let server_address = AddressStub::new(0);
	let mut server = UdpServer::new(transport.create_channel(server_address.clone()));
	
	let private_key = create_user_private_key_stub();
	let public_key = create_user_public_key_stub();
	server.add_allowed_user(private_key, public_key);
	
	let client_address = AddressStub::new(1);
	let mut client = UdpClient::new(private_key, public_key, transport.create_channel(client_address), server_address);
	(server, public_key, client, transport)
}


pub fn create_user_private_key_stub() -> UserPrivateKey {
	let mut result = [0; 32];
	OsRng.fill_bytes(&mut result);
	result
}


pub fn create_user_public_key_stub() -> UserPublicKey {
	let mut result = [0; 4];
	OsRng.fill_bytes(&mut result);
	result
}


