use std::collections::VecDeque;

use crate::protocol::frame::applications::{BothDirectionCommand, ChannelSequence, CommandWithChannel};
use crate::protocol::frame::channel::{Channel, ChannelType};
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::MAX_COMMAND_IN_FRAME;

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Debug)]
pub struct OutCommandsCollector {
	pub commands: VecDeque<CommandWithChannel>,
	group_sequence: [ChannelSequence; 256],
}

#[derive(Debug)]
pub struct CommandWithChannelType {
	pub channel_type: ChannelType,
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
	pub fn add_command(&mut self, channel_type: ChannelType, command: BothDirectionCommand) {
		match self.create_channel(&channel_type) {
			None => {
				tracing::error!("can not create channel for {:?} {:?}", channel_type, command)
			}
			Some(channel) => {
				self.commands.push_back(CommandWithChannel {
					channel,
					both_direction_command: command,
				});
			}
		}
	}

	fn create_channel(&mut self, channel_type: &ChannelType) -> Option<Channel> {
		match channel_type {
			ChannelType::ReliableUnordered => Option::Some(Channel::ReliableUnordered),
			ChannelType::ReliableOrdered(group_id) => Option::Some(Channel::ReliableOrdered(*group_id)),
			ChannelType::UnreliableUnordered => Option::Some(Channel::UnreliableUnordered),
			ChannelType::UnreliableOrdered(group_id) => Option::Some(Channel::UnreliableOrdered(*group_id)),
			ChannelType::ReliableSequence(group) => {
				let mut sequence = &mut self.group_sequence[group.0 as usize];
				let result = Option::Some(Channel::ReliableSequence(*group, *sequence));
				sequence.0 += 1;
				result
			}
		}
	}

	pub fn contains_self_data(&self) -> bool {
		!self.commands.is_empty()
	}

	pub fn build_frame(&mut self, frame: &mut OutFrame) {
		let mut command_count = 0;
		while let Some(command) = self.commands.pop_front() {
			if let Err(()) = frame.add_command(command) {
				break;
			}
			command_count += 1;
			if command_count == MAX_COMMAND_IN_FRAME {
				break;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::protocol::commands::output::OutCommandsCollector;
	use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup};
	use crate::protocol::frame::channel::{Channel, ChannelType};
	use crate::protocol::frame::output::OutFrame;
	use crate::protocol::frame::MAX_COMMAND_IN_FRAME;

	#[test]
	pub fn test_group_sequence() {
		let mut output = OutCommandsCollector::default();
		for _ in 0..3 {
			output.add_command(
				ChannelType::ReliableSequence(ChannelGroup(100)),
				BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			);
		}
		assert!(matches!(output.commands[0].channel, Channel::ReliableSequence(_,sequence)
			if sequence.0==0));
		assert!(matches!(output.commands[1].channel, Channel::ReliableSequence(_,sequence)
			if sequence.0==1));
		assert!(matches!(output.commands[2].channel, Channel::ReliableSequence(_,sequence)
			if sequence.0==2));
	}

	#[test]
	pub fn should_split_commands() {
		let mut output = OutCommandsCollector::default();
		for i in 0..2 * MAX_COMMAND_IN_FRAME {
			output.add_command(
				ChannelType::ReliableSequence(ChannelGroup(100)),
				BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
					object_id: Default::default(),
					field_id: 1,
					value: i as i64,
				})),
			);
		}

		let mut frame = OutFrame::new(0);
		output.build_frame(&mut frame);

		// в коллекторе первой должна быть команда с value равным размеру фрейма
		assert!(matches!(
			output.commands.pop_front().unwrap().both_direction_command,
			BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
					object_id: _,
					field_id: _,
					value,
				}))
			if value == MAX_COMMAND_IN_FRAME as i64
		));

		// проверяем как собран фрейм
		for i in 0..MAX_COMMAND_IN_FRAME {
			assert!(matches!(
				frame.get_commands().as_slice()[i].both_direction_command,
				BothDirectionCommand::C2S( C2SCommand::SetLong(SetLongCommand {
						object_id: _,
						field_id: _,
						value,
					}))
				if value == i as i64
			));
		}
	}
}
