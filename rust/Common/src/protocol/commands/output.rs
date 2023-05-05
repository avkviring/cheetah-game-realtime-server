use std::collections::VecDeque;

use crate::protocol::frame::applications::{BothDirectionCommand, ChannelSequence, CommandWithReliabilityGuarantees};
use crate::protocol::frame::channel::{ReliabilityGuarantees, ReliabilityGuaranteesChannel};
use crate::protocol::frame::output::OutFrame;

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Debug)]
pub struct OutCommandsCollector {
	pub commands: VecDeque<CommandWithReliabilityGuarantees>,
	group_sequence: [ChannelSequence; 256],
}

#[derive(Debug)]
pub struct CommandWithChannelType {
	pub channel_type: ReliabilityGuarantees,
	pub command: BothDirectionCommand,
}

impl Default for OutCommandsCollector {
	fn default() -> Self {
		Self {
			commands: VecDeque::with_capacity(64),
			group_sequence: [ChannelSequence(0); 256],
		}
	}
}

impl OutCommandsCollector {
	pub fn add_command(&mut self, channel_type: ReliabilityGuarantees, command: BothDirectionCommand) {
		match self.create_channel(channel_type) {
			None => {
				tracing::error!("can not create channel for {:?} {:?}", channel_type, command);
			}
			Some(channel) => {
				self.commands.push_back(CommandWithReliabilityGuarantees {
					reliability_guarantees: channel,
					commands: command,
				});
			}
		}
	}

	fn create_channel(&mut self, channel_type: ReliabilityGuarantees) -> Option<ReliabilityGuaranteesChannel> {
		match channel_type {
			ReliabilityGuarantees::ReliableUnordered => Some(ReliabilityGuaranteesChannel::ReliableUnordered),
			ReliabilityGuarantees::ReliableOrdered(group_id) => Some(ReliabilityGuaranteesChannel::ReliableOrdered(group_id)),
			ReliabilityGuarantees::UnreliableUnordered => Some(ReliabilityGuaranteesChannel::UnreliableUnordered),
			ReliabilityGuarantees::UnreliableOrdered(group_id) => Some(ReliabilityGuaranteesChannel::UnreliableOrdered(group_id)),
			ReliabilityGuarantees::ReliableSequence(group) => {
				let mut sequence = &mut self.group_sequence[group.0 as usize];
				let result = Some(ReliabilityGuaranteesChannel::ReliableSequence(group, *sequence));
				sequence.0 += 1;
				result
			}
		}
	}

	#[must_use]
	pub fn contains_self_data(&self) -> bool {
		!self.commands.is_empty()
	}

	pub fn build_frame(&mut self, frame: &mut OutFrame) {
		while let Some(command) = self.commands.pop_front() {
			if !frame.add_command(command.clone()) {
				self.commands.push_front(command);
				break;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use crate::commands::binary_value::Buffer;
	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::event::EventCommand;
	use crate::protocol::commands::output::OutCommandsCollector;
	use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup};
	use crate::protocol::frame::channel::{ReliabilityGuarantees, ReliabilityGuaranteesChannel};
	use crate::protocol::frame::output::OutFrame;
	use crate::protocol::frame::MAX_FRAME_SIZE;

	#[test]
	pub(crate) fn test_group_sequence() {
		let mut output = OutCommandsCollector::default();
		for _ in 0..3 {
			output.add_command(ReliabilityGuarantees::ReliableSequence(ChannelGroup(100)), BothDirectionCommand::C2S(C2SCommand::AttachToRoom));
		}
		assert!(matches!(output.commands[0].reliability_guarantees, ReliabilityGuaranteesChannel::ReliableSequence(_,sequence)
			if sequence.0==0));
		assert!(matches!(output.commands[1].reliability_guarantees, ReliabilityGuaranteesChannel::ReliableSequence(_,sequence)
			if sequence.0==1));
		assert!(matches!(output.commands[2].reliability_guarantees, ReliabilityGuaranteesChannel::ReliableSequence(_,sequence)
			if sequence.0==2));
	}

	#[test]
	pub(crate) fn should_split_commands_by_size() {
		let mut output = OutCommandsCollector::default();
		for _i in 0..MAX_FRAME_SIZE {
			output.add_command(
				ReliabilityGuarantees::ReliableSequence(ChannelGroup(100)),
				BothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 1,
					event: Buffer::from([1, 2, 3, 4].as_slice()),
				})),
			);
		}
		let output_commands = output.commands.clone();
		let mut frames_commands = VecDeque::new();

		loop {
			let mut first_frame = OutFrame::new(0, 0);
			output.build_frame(&mut first_frame);
			let iter = first_frame.get_commands();
			if iter.len() == 0 {
				break;
			}
			iter.for_each(|c| frames_commands.push_back(c.clone()));
		}

		assert_eq!(output_commands, frames_commands);
	}
}
