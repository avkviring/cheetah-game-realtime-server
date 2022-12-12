use std::time::Instant;
use prometheus::IntCounter;

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::protocol::frame::applications::BothDirectionCommand;
use cheetah_common::protocol::frame::channel::ChannelType;
use cheetah_common::protocol::Protocol;

use crate::stub::Channel;

pub mod stub;

///
/// Тестирование отправки команд с клиента на сервер
///
#[test]
fn should_send_from_client() {
	let mut peer_a = Protocol::new(Instant::now(), Instant::now(), IntCounter::new("name", "help").unwrap().local());
	let mut peer_b = Protocol::new(Instant::now(), Instant::now(), IntCounter::new("name", "help").unwrap().local());

	peer_a
		.out_commands_collector
		.add_command(ChannelType::ReliableUnordered, BothDirectionCommand::C2S(C2SCommand::AttachToRoom));

	peer_a
		.out_commands_collector
		.add_command(ChannelType::UnreliableUnordered, BothDirectionCommand::C2S(C2SCommand::DetachFromRoom));

	let mut channel = Channel::default();
	channel.cycle(1, &mut peer_a, &mut peer_b);

	let commands = peer_b.in_commands_collector.get_ready_commands();

	assert!(commands
		.iter()
		.any(|p| p.both_direction_command == BothDirectionCommand::C2S(C2SCommand::AttachToRoom)));
	assert!(commands
		.iter()
		.any(|p| p.both_direction_command == BothDirectionCommand::C2S(C2SCommand::DetachFromRoom)));
}

///
/// Тестирование надежной доставки по ненадежному каналу
#[test]
fn should_transfer_reliable_on_unreliable_channel() {
	let mut peer_a = Protocol::new(Instant::now(), Instant::now(), IntCounter::new("name", "help").unwrap().local());
	let mut peer_b = Protocol::new(Instant::now(), Instant::now(), IntCounter::new("name", "help").unwrap().local());

	peer_a
		.out_commands_collector
		.add_command(ChannelType::ReliableUnordered, BothDirectionCommand::C2S(C2SCommand::AttachToRoom));

	peer_a
		.out_commands_collector
		.add_command(ChannelType::UnreliableUnordered, BothDirectionCommand::C2S(C2SCommand::DetachFromRoom));

	let mut channel = Channel::default();
	channel.add_reliable_percent(0..=10, 0.0);
	channel.cycle(1, &mut peer_a, &mut peer_b);

	let commands = peer_b.in_commands_collector.get_ready_commands();
	assert!(commands.is_empty());

	channel.cycle(15, &mut peer_a, &mut peer_b);

	let commands = peer_b.in_commands_collector.get_ready_commands();
	assert_eq!(commands.len(), 1);
	assert!(commands
		.iter()
		.any(|p| p.both_direction_command == BothDirectionCommand::C2S(C2SCommand::AttachToRoom)));
}
