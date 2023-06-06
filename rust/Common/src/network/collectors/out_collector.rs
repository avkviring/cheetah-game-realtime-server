use std::collections::VecDeque;

use fnv::FnvHashMap;

use cheetah_protocol::OutputDataProducer;

use crate::commands::codec::encoder::encode_commands;
use crate::commands::guarantees::{ChannelGroup, ChannelSequence, ReliabilityGuarantees, ReliabilityGuaranteesChannel};
use crate::commands::{BothDirectionCommand, CommandWithReliabilityGuarantees};

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Debug, Default)]
pub struct OutCommandsCollector {
	sequences: FnvHashMap<SequenceKey, ChannelSequence>,
	commands: VecDeque<CommandWithReliabilityGuarantees>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct SequenceKey(pub ReliabilityGuarantees, pub ChannelGroup);

impl OutputDataProducer for OutCommandsCollector {
	fn contains_output_data(&self) -> bool {
		!self.commands.is_empty()
	}

	fn get_output_data(&mut self, packet: &mut [u8]) -> (usize, bool) {
		encode_commands(&mut self.commands, packet)
	}
}

impl OutCommandsCollector {
	pub fn add_command(&mut self, channel_type: ReliabilityGuarantees, command: BothDirectionCommand) {
		match self.create_channel(channel_type) {
			None => {
				tracing::error!("can not create channel for {:?} {:?}", channel_type, command);
			}
			Some(channel) => {
				let command = CommandWithReliabilityGuarantees {
					reliability_guarantees: channel,
					command,
				};
				self.commands.push_back(command);
			}
		}
	}

	fn create_channel(&mut self, channel_type: ReliabilityGuarantees) -> Option<ReliabilityGuaranteesChannel> {
		match channel_type {
			ReliabilityGuarantees::ReliableUnordered => Some(ReliabilityGuaranteesChannel::ReliableUnordered),
			ReliabilityGuarantees::ReliableOrdered(group) => Some(ReliabilityGuaranteesChannel::ReliableOrdered(group, self.next_sequence(channel_type, group))),
			ReliabilityGuarantees::UnreliableUnordered => Some(ReliabilityGuaranteesChannel::UnreliableUnordered),
			ReliabilityGuarantees::UnreliableOrdered(group) => Some(ReliabilityGuaranteesChannel::UnreliableOrdered(group, self.next_sequence(channel_type, group))),
			ReliabilityGuarantees::ReliableSequence(group) => Some(ReliabilityGuaranteesChannel::ReliableSequence(group, self.next_sequence(channel_type, group))),
		}
	}

	fn next_sequence(&mut self, guarantees: ReliabilityGuarantees, group: ChannelGroup) -> ChannelSequence {
		let key = SequenceKey(guarantees, group);
		let mut sequence = self.sequences.get_mut(&key);
		if sequence.is_none() {
			self.sequences.insert(key.clone(), Default::default());
			sequence = self.sequences.get_mut(&key);
		}
		let channel_sequence = sequence.unwrap();
		let result = *channel_sequence;
		channel_sequence.0 += 1;
		result
	}
}

#[cfg(test)]
mod tests {
	use cheetah_protocol::frame::packets_collector::PACKET_SIZE;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::c2s::C2SCommand::IncrementLongValue;
	use crate::commands::codec::decoder::decode_commands;
	use crate::commands::guarantees::{ChannelGroup, ChannelSequence, ReliabilityGuarantees};
	use crate::commands::types::long::IncrementLongC2SCommand;
	use crate::commands::BothDirectionCommand;
	use crate::network::collectors::out_collector::*;
	use crate::room::field::FieldId;

	#[test]
	fn test_group_sequence() {
		let mut collector = OutCommandsCollector::default();
		let group = ChannelGroup(100);
		for _ in 0..3 {
			collector.add_command(ReliabilityGuarantees::ReliableSequence(group), BothDirectionCommand::C2S(C2SCommand::AttachToRoom));
		}

		let key = SequenceKey(ReliabilityGuarantees::ReliableSequence(group), group);
		assert_eq!(collector.sequences.get(&key).unwrap(), &ChannelSequence(3));
	}

	#[test]
	fn should_not_contains_data() {
		let collector = OutCommandsCollector::default();
		assert!(!collector.contains_output_data());
	}

	#[test]
	fn should_contains_data() {
		let mut collector = OutCommandsCollector::default();
		collector.add_command(ReliabilityGuarantees::UnreliableUnordered, BothDirectionCommand::C2S(C2SCommand::AttachToRoom));
		assert!(collector.contains_output_data());
	}

	#[test]
	fn should_not_contains_data_after_get_data() {
		let mut collector = OutCommandsCollector::default();
		collector.add_command(ReliabilityGuarantees::UnreliableUnordered, BothDirectionCommand::C2S(C2SCommand::AttachToRoom));
		let mut packet = [0; PACKET_SIZE];
		collector.get_output_data(&mut packet);
		assert!(!collector.contains_output_data());
	}

	#[test]
	fn should_correct_decode() {
		let mut collector = OutCommandsCollector::default();
		collector.add_command(ReliabilityGuarantees::UnreliableUnordered, BothDirectionCommand::C2S(C2SCommand::AttachToRoom));
		let mut packet = [0; PACKET_SIZE];
		let (size, _) = collector.get_output_data(&mut packet);
		let commands = decode_commands(true, &packet[0..size]).unwrap();
		assert_eq!(
			commands,
			vec![CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::UnreliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			}]
		);
	}

	#[test]
	fn should_ordered_commands() {
		let mut collector = OutCommandsCollector::default();
		collector.add_command(ReliabilityGuarantees::UnreliableUnordered, create_command(0));
		collector.add_command(ReliabilityGuarantees::UnreliableUnordered, create_command(1));

		let commands = send_and_receive(&mut collector);

		assert_eq!(
			commands[0],
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::UnreliableUnordered,
				command: create_command(0),
			}
		);
		assert_eq!(
			commands[1],
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::UnreliableUnordered,
				command: create_command(1),
			}
		);
	}

	#[test]
	fn should_correct_decode_many_commands() {
		const COUNT: usize = 1000;
		let mut collector = OutCommandsCollector::default();
		for i in 0..COUNT {
			collector.add_command(ReliabilityGuarantees::UnreliableUnordered, create_command(i));
		}
		let commands = send_and_receive(&mut collector);

		for i in 0..COUNT {
			assert_eq!(
				commands[i],
				CommandWithReliabilityGuarantees {
					reliability_guarantees: ReliabilityGuaranteesChannel::UnreliableUnordered,
					command: create_command(i),
				}
			);
		}
	}

	fn send_and_receive(collector: &mut OutCommandsCollector) -> Vec<CommandWithReliabilityGuarantees> {
		let mut commands = Vec::new();
		while collector.contains_output_data() {
			let mut packet = [0; PACKET_SIZE];
			let (size, _) = collector.get_output_data(&mut packet);
			for command in decode_commands(true, &packet[0..size]).unwrap() {
				commands.push(command);
			}
		}
		commands
	}

	fn create_command(i: usize) -> BothDirectionCommand {
		BothDirectionCommand::C2S(IncrementLongValue(IncrementLongC2SCommand {
			object_id: Default::default(),
			field_id: i as FieldId,
			increment: 0,
		}))
	}
}
