use std::net::SocketAddr;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, Instant};

use cheetah_relay_common::udp::channel::{Channel, Transport, TransportError, UDPTransport};
use cheetah_relay_common::udp::client::UdpClient;
use cheetah_relay_common::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
use cheetah_relay_common::udp::server::UdpServer;

use crate::udp::stub::{create_user_private_key_stub, create_user_public_key_stub, new_ping_command};

#[test]
fn should_send_throught_udp() {
	let transport = UDPTransport::default();
	let (mut server, public_key, mut client) = setup_udp(transport);
	
	client.protocol.out_commands_collector.add_reliability_command(new_ping_command("test reliability".to_string()));
	client.protocol.out_commands_collector.add_unreliability_command(new_ping_command("test unreliability".to_string()));
	
	let now = Instant::now();
	for _ in 0..10 {
		client.cycle(&now);
		server.cycle(&now);
		thread::sleep(Duration::from_millis(10));
	}
	
	let protocol = &mut server.get_user_sessions(&public_key).protocol;
	let commands = protocol.in_commands_collector.get_commands();
	
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::TestSimple(v) if *v == "test reliability".to_string())).is_some());
	assert!(commands.iter().find(|p| matches!(p, ApplicationCommand::TestSimple(v) if *v == "test unreliability".to_string())).is_some());
}


fn setup_udp(transport: UDPTransport) -> (UdpServer<SocketAddr>, [u8; 4], UdpClient<SocketAddr>) {
	for server_port in 2000..4000 {
		let server_address = SocketAddr::from_str(format!("127.0.0.1:{}", server_port).as_str()).unwrap();
		match transport.create_channel(server_address.clone()) {
			Ok(channel) => {
				let mut server = UdpServer::new(channel);
				let private_key = create_user_private_key_stub();
				let public_key = create_user_public_key_stub();
				server.add_allowed_user(private_key, public_key);
				
				
				for client_port in 4000..7000 {
					let client_address = SocketAddr::from_str(format!("127.0.0.1:{}", client_port).as_str()).unwrap();
					
					match transport.create_channel(client_address) {
						Ok(channel) => {
							let mut client = UdpClient::new(private_key, public_key, channel, server_address);
							return (server, public_key, client);
						}
						Err(_) => {}
					}
				}
			}
			Err(_) => {}
		}
	}
	
	panic!("all port in use");
}