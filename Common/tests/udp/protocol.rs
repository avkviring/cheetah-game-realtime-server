use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
use cheetah_relay_common::protocol::relay::RelayProtocol;

use crate::udp::stub::Channel;
use std::time::Instant;

///
/// Тестирование отправки команд с клиента на сервер
///
#[test]
fn should_send_from_client() {
	let mut peer_a = RelayProtocol::new(&Instant::now());
	let mut peer_b = RelayProtocol::new(&Instant::now());
	
	peer_a
		.out_commands_collector
		.add_command(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::TestSimple("test reliability".to_string()),
		});
	peer_a
		.out_commands_collector
		.add_command(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::UnreliableUnordered,
			command: ApplicationCommand::TestSimple("test unreliability".to_string()),
		});
	
	let mut channel = Channel::default();
	channel.cycle(1, &mut peer_a, &mut peer_b);
	
	
	let commands = peer_b.in_commands_collector.get_commands();
	
	assert!(commands.iter().find(|p| matches!(&p.command, ApplicationCommand::TestSimple(v) if *v == "test reliability".to_string())).is_some());
	assert!(commands.iter().find(|p| matches!(&p.command, ApplicationCommand::TestSimple(v) if *v == "test unreliability".to_string())).is_some());
}

///
/// Тестирование надежной доставки по ненадежному каналу
#[test]
fn should_transfer_reliable_on_unreliable_channel() {
	let mut peer_a = RelayProtocol::new(&Instant::now());
	let mut peer_b = RelayProtocol::new(&Instant::now());
	
	peer_a
		.out_commands_collector
		.add_command(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::TestSimple("test reliability".to_string()),
		});
	peer_a
		.out_commands_collector
		.add_command(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::UnreliableUnordered,
			command: ApplicationCommand::TestSimple("test unreliability".to_string()),
		});
	
	let mut channel = Channel::default();
	channel.add_reliable_percent(0..=10, 0.0);
	channel.cycle(1, &mut peer_a, &mut peer_b);
	
	
	let commands = peer_b.in_commands_collector.get_commands();
	assert!(commands.is_empty());
	
	channel.cycle(15, &mut peer_a, &mut peer_b);
	
	let commands = peer_b.in_commands_collector.get_commands();
	assert_eq!(commands.len(), 1);
	assert!(commands.iter().find(|p| matches!(&p.command, ApplicationCommand::TestSimple(v) if *v == "test reliability".to_string())).is_some());
}
