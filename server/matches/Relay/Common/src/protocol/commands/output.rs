use std::collections::HashMap;
use std::time::Instant;

use fnv::FnvBuildHasher;

use crate::protocol::frame::applications::{
	BothDirectionCommand, CommandWithChannel, ApplicationCommands, ChannelGroup, ChannelSequence,
};
use crate::protocol::frame::channel::{CommandChannel, ApplicationCommandChannelType};
use crate::protocol::frame::Frame;
use crate::protocol::FrameBuilder;
use crate::room::object::GameObjectId;

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Default, Debug)]
pub struct OutCommandsCollector {
	pub commands: ApplicationCommands,
	group_sequence: HashMap<ChannelGroup, ChannelSequence, FnvBuildHasher>,
	object_sequence: HashMap<GameObjectId, ChannelSequence, FnvBuildHasher>,
}

impl OutCommandsCollector {
	const MAX_COMMAND_IN_FRAME: usize = 3;

	pub fn add_command(&mut self, channel_type: ApplicationCommandChannelType, command: BothDirectionCommand) {
		match self.create_channel(&channel_type, &command) {
			None => {
				log::error!("can not create channel for {:?} {:?}", channel_type, command)
			}
			Some(channel) => {
				let description = CommandWithChannel { channel, command };
				let commands = match channel_type {
					ApplicationCommandChannelType::ReliableUnordered
					| ApplicationCommandChannelType::ReliableOrderedByObject
					| ApplicationCommandChannelType::ReliableOrderedByGroup(_)
					| ApplicationCommandChannelType::ReliableSequenceByObject
					| ApplicationCommandChannelType::ReliableSequenceByGroup(_) => &mut self.commands.reliable,

					ApplicationCommandChannelType::UnreliableUnordered
					| ApplicationCommandChannelType::UnreliableOrderedByObject
					| ApplicationCommandChannelType::UnreliableOrderedByGroup(_) => &mut self.commands.unreliable,
				};

				commands.push_back(description);
			}
		}
	}

	fn create_channel(
        &mut self,
        channel_type: &ApplicationCommandChannelType,
        command: &BothDirectionCommand,
	) -> Option<CommandChannel> {
		match channel_type {
			ApplicationCommandChannelType::ReliableUnordered => Option::Some(CommandChannel::ReliableUnordered),
			ApplicationCommandChannelType::ReliableOrderedByObject => {
				Option::Some(CommandChannel::ReliableOrderedByObject)
			}
			ApplicationCommandChannelType::ReliableOrderedByGroup(group_id) => {
				Option::Some(CommandChannel::ReliableOrderedByGroup(*group_id))
			}
			ApplicationCommandChannelType::UnreliableUnordered => Option::Some(CommandChannel::UnreliableUnordered),
			ApplicationCommandChannelType::UnreliableOrderedByObject => {
				Option::Some(CommandChannel::UnreliableOrderedByObject)
			}
			ApplicationCommandChannelType::UnreliableOrderedByGroup(group_id) => {
				Option::Some(CommandChannel::UnreliableOrderedByGroup(*group_id))
			}
			ApplicationCommandChannelType::ReliableSequenceByObject => command.get_object_id().and_then(|game_object_id| {
				let sequence = self
					.object_sequence
					.entry(game_object_id.clone())
					.and_modify(|v| *v += 1)
					.or_insert(0);
				Option::Some(CommandChannel::ReliableSequenceByObject(sequence.clone()))
			}),
			ApplicationCommandChannelType::ReliableSequenceByGroup(group) => {
				let sequence = self.group_sequence.entry(*group).and_modify(|v| *v += 1).or_insert(0);
				Option::Some(CommandChannel::ReliableSequenceByGroup(*group, sequence.clone()))
			}
		}
	}
}

impl FrameBuilder for OutCommandsCollector {
	fn contains_self_data(&self, _: &Instant) -> bool {
		self.commands.reliable.len() + self.commands.unreliable.len() > 0
	}

	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		let mut command_count = 0;
		while let Some(command) = self.commands.reliable.pop_front() {
			frame.commands.reliable.push_back(command);
			command_count += 1;
			if command_count == OutCommandsCollector::MAX_COMMAND_IN_FRAME {
				break;
			}
		}
		if command_count == OutCommandsCollector::MAX_COMMAND_IN_FRAME {
			return;
		}

		while let Some(command) = self.commands.unreliable.pop_front() {
			frame.commands.unreliable.push_back(command);
			command_count += 1;
			if command_count == OutCommandsCollector::MAX_COMMAND_IN_FRAME {
				break;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::Instant;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::event::EventCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::protocol::commands::output::OutCommandsCollector;
	use crate::protocol::frame::applications::BothDirectionCommand;
	use crate::protocol::frame::channel::{CommandChannel, ApplicationCommandChannelType};
	use crate::protocol::frame::Frame;
	use crate::protocol::FrameBuilder;

	#[test]
	pub fn test_group_sequence() {
		let mut output = OutCommandsCollector::default();
		for _ in 0..3 {
			output.add_command(
                ApplicationCommandChannelType::ReliableSequenceByGroup(100),
                BothDirectionCommand::C2SCommand(C2SCommand::AttachToRoom),
			);
		}
		assert!(
			matches!(output.commands.reliable[0].channel, CommandChannel::ReliableSequenceByGroup(_,sequence) if sequence==0)
		);
		assert!(
			matches!(output.commands.reliable[1].channel, CommandChannel::ReliableSequenceByGroup(_,sequence) if sequence==1)
		);
		assert!(
			matches!(output.commands.reliable[2].channel, CommandChannel::ReliableSequenceByGroup(_,sequence) if sequence==2)
		);
	}

	#[test]
	pub fn should_split_commands() {
		let mut output = OutCommandsCollector::default();
		for i in 0..2 * OutCommandsCollector::MAX_COMMAND_IN_FRAME {
			output.add_command(
                ApplicationCommandChannelType::ReliableSequenceByGroup(100),
                BothDirectionCommand::C2SCommand(C2SCommand::SetLong(SetLongCommand {
					object_id: Default::default(),
					field_id: 1,
					value: i as i64,
				})),
			);
		}

		let mut frame = Frame::new(0);
		output.build_frame(&mut frame, &Instant::now());

		// в коллекторе первой должна быть команда с value равным размеру фрейма
		assert!(matches!(
			output.commands.reliable.pop_front().unwrap().command,
			BothDirectionCommand::C2SCommand(C2SCommand::SetLong(SetLongCommand {
					object_id: _,
					field_id: _,
					value,
				}))
			if value == OutCommandsCollector::MAX_COMMAND_IN_FRAME as i64
		));

		// проверяем как собран фрейм
		for i in 0..OutCommandsCollector::MAX_COMMAND_IN_FRAME {
			assert!(matches!(
				frame.commands.reliable.pop_front().unwrap().command,
				BothDirectionCommand::C2SCommand( C2SCommand::SetLong(SetLongCommand {
						object_id: _,
						field_id: _,
						value,
					}))
				if value == i as i64
			));
		}
	}

	#[test]
	pub fn test_object_sequence() {
		let mut output = OutCommandsCollector::default();

		for _ in 0..3 {
			output.add_command(
                ApplicationCommandChannelType::ReliableSequenceByObject,
                BothDirectionCommand::C2SCommand(C2SCommand::Event(EventCommand {
					object_id: Default::default(),
					field_id: 0,
					event: Default::default(),
				})),
			);
		}

		assert!(
			matches!(output.commands.reliable[0].channel, CommandChannel::ReliableSequenceByObject(sequence) if sequence==0)
		);
		assert!(
			matches!(output.commands.reliable[1].channel, CommandChannel::ReliableSequenceByObject(sequence) if sequence==1)
		);
		assert!(
			matches!(output.commands.reliable[2].channel, CommandChannel::ReliableSequenceByObject(sequence) if sequence==2)
		);
	}
}
