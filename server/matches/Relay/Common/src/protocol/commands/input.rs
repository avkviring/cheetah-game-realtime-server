use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence, CommandWithChannel};
use crate::protocol::frame::channel::Channel;
use crate::protocol::frame::{Frame, FrameId};

///
/// Коллектор входящих команд
/// - поддержка мультиплексирования
///
#[derive(Debug)]
pub struct InCommandsCollector {
	ordered: [FrameId; 256],
	sequences: [ChannelSequence; 256],
	sequence_commands: [Option<BinaryHeap<SequenceApplicationCommand>>; 256],
	ready_commands: Vec<CommandWithChannel>,
	is_get_ready_commands: bool,
}

///
/// Лимит очереди для команд ожидающих сбора последовательности для одной группы
/// Применяется для исключения атаки на память сервера путем посылки с клиента никогда не
/// завершающихся последовательностей
///
const SEQUENCE_COMMANDS_LIMIT: usize = 256;

impl Default for InCommandsCollector {
	fn default() -> Self {
		Self {
			ordered: [0; 256],
			sequences: [ChannelSequence(0); 256],
			sequence_commands: [(); 256].map(|_| Option::None),
			ready_commands: Default::default(),
			is_get_ready_commands: false,
		}
	}
}

impl InCommandsCollector {
	pub fn get_ready_commands(&mut self) -> &[CommandWithChannel] {
		if self.is_get_ready_commands {
			self.ready_commands.clear();
		}
		self.is_get_ready_commands = true;
		self.ready_commands.as_slice()
	}

	pub fn collect(&mut self, frame: Frame) {
		if self.is_get_ready_commands {
			self.ready_commands.clear();
			self.is_get_ready_commands = false;
		}

		let frame_id = frame.frame_id;
		frame.commands.into_iter().for_each(|c| {
			match c.channel {
				Channel::ReliableUnordered | Channel::UnreliableUnordered => self.ready_commands.push(c),
				Channel::ReliableOrdered(group) | Channel::UnreliableOrdered(group) => {
					self.process_ordered(group, frame_id, c);
				}
				Channel::ReliableSequence(channel_id, sequence) => self.process_sequence(channel_id, sequence, c),
			};
		});
	}

	fn process_sequence(&mut self, channel_group: ChannelGroup, input_sequence: ChannelSequence, command: CommandWithChannel) {
		let mut is_ready_command = false;
		let allow_sequence = &mut self.sequences[channel_group.0 as usize];

		if input_sequence == ChannelSequence::FIRST || input_sequence == *allow_sequence {
			self.ready_commands.push(command.clone());
			*allow_sequence = input_sequence.next();
			is_ready_command = true;
		}

		let mut current_ready_sequence = *allow_sequence;
		if let Some(commands) = &mut self.sequence_commands[channel_group.0 as usize] {
			while let Option::Some(command_with_sequence) = commands.peek() {
				let command_sequence = command_with_sequence.sequence;
				if command_sequence == current_ready_sequence {
					self.ready_commands.push(commands.pop().unwrap().command);
					current_ready_sequence = command_sequence.next();
					*allow_sequence = current_ready_sequence;
				} else {
					break;
				}
			}
		}

		if !is_ready_command {
			if self.sequence_commands[channel_group.0 as usize].is_none() {
				self.sequence_commands[channel_group.0 as usize] = Option::Some(BinaryHeap::default());
			}
			let option_buffer = &mut self.sequence_commands[channel_group.0 as usize];
			let buffer = option_buffer.as_mut().unwrap();
			if buffer.len() > SEQUENCE_COMMANDS_LIMIT {
				log::error!("Sequence commands buffer overflow")
			} else {
				buffer.push(SequenceApplicationCommand {
					sequence: input_sequence,
					command,
				});
			}
		}
	}

	fn process_ordered(&mut self, channel_group: ChannelGroup, frame_id: FrameId, command: CommandWithChannel) {
		let order = &self.ordered[channel_group.0 as usize];
		if frame_id >= *order {
			self.ordered[channel_group.0 as usize] = frame_id;
			self.ready_commands.push(command);
		}
	}
}

#[derive(Debug)]
struct SequenceApplicationCommand {
	sequence: ChannelSequence,
	command: CommandWithChannel,
}

impl PartialEq for SequenceApplicationCommand {
	fn eq(&self, other: &Self) -> bool {
		self.sequence.eq(&other.sequence)
	}
}

impl PartialOrd for SequenceApplicationCommand {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Option::Some(self.cmp(other))
	}
}

impl Eq for SequenceApplicationCommand {}

impl Ord for SequenceApplicationCommand {
	fn cmp(&self, other: &Self) -> Ordering {
		self.sequence.0.cmp(&other.sequence.0).reverse()
	}

	fn max(self, other: Self) -> Self
	where
		Self: Sized,
	{
		if self.sequence.0 > other.sequence.0 {
			self
		} else {
			other
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::protocol::commands::input::InCommandsCollector;
	use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, ChannelSequence, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::{Frame, FrameId};
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	pub fn test_clear_after_get_ready_commands() {
		let mut in_commands = InCommandsCollector::default();
		let cmd_1 = create_test_command(Channel::ReliableUnordered, 1);
		let mut frame = Frame::new(1);
		frame.commands.push(cmd_1.clone()).unwrap();
		in_commands.collect(frame);
		assert_eq!(in_commands.get_ready_commands(), [cmd_1]);
		assert_eq!(in_commands.get_ready_commands(), []);
	}

	#[test]
	pub fn test_not_clear_after_collect() {
		let mut in_commands = InCommandsCollector::default();
		let cmd_1 = create_test_command(Channel::ReliableUnordered, 1);
		let mut frame = Frame::new(1);
		frame.commands.push(cmd_1.clone()).unwrap();
		in_commands.collect(frame.clone());
		in_commands.collect(frame);
		assert_eq!(in_commands.get_ready_commands(), [cmd_1.clone(), cmd_1]);
		assert_eq!(in_commands.get_ready_commands(), []);
	}

	#[test]
	pub fn test_unordered() {
		let mut in_commands = InCommandsCollector::default();
		let cmd_1 = create_test_command(Channel::ReliableUnordered, 1);
		let cmd_2 = create_test_command(Channel::ReliableUnordered, 2);

		assert(2, &mut in_commands, &[cmd_2.clone()], &[cmd_2]);
		assert(1, &mut in_commands, &[cmd_1.clone()], &[cmd_1]);
	}

	#[test]
	pub fn test_group_ordered() {
		let mut in_commands = InCommandsCollector::default();

		let cmd_1 = create_test_command(Channel::ReliableOrdered(ChannelGroup(1)), 1);
		let cmd_2 = create_test_command(Channel::ReliableOrdered(ChannelGroup(1)), 2);
		let cmd_3 = create_test_command(Channel::ReliableOrdered(ChannelGroup(1)), 3);

		assert(1, &mut in_commands, &[cmd_1.clone()], &[cmd_1]);
		assert(3, &mut in_commands, &[cmd_3.clone()], &[cmd_3]);
		assert(2, &mut in_commands, &[cmd_2], &[]);
	}

	#[test]
	pub fn test_group_ordered_when_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let cmd_1 = create_test_command(Channel::ReliableOrdered(ChannelGroup(1)), 1);
		let cmd_2 = create_test_command(Channel::ReliableOrdered(ChannelGroup(2)), 2);

		assert(2, &mut in_commands, &[cmd_2.clone()], &[cmd_2]);
		assert(1, &mut in_commands, &[cmd_1.clone()], &[cmd_1]);
	}

	#[test]
	pub fn test_group_sequence() {
		let mut in_commands = InCommandsCollector::default();

		let cmd_1 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(0)), 1);
		let cmd_2 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(1)), 2);
		let cmd_3 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(2)), 3);
		let cmd_4 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(3)), 4);
		let cmd_5 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(4)), 5);
		let cmd_6 = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(5)), 5);

		assert(3, &mut in_commands, &[cmd_3.clone()], &[]);
		assert(1, &mut in_commands, &[cmd_1.clone()], &[cmd_1]);
		assert(5, &mut in_commands, &[cmd_5.clone()], &[]);
		assert(2, &mut in_commands, &[cmd_2.clone()], &[cmd_2, cmd_3]);
		assert(4, &mut in_commands, &[cmd_4.clone()], &[cmd_4, cmd_5]);
		assert(6, &mut in_commands, &[cmd_6.clone()], &[cmd_6]);
	}

	#[test]
	pub fn test_group_sequence_with_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let cmd_1_a = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(0)), 1);
		let cmd_1_b = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(1)), 2);
		let cmd_1_c = create_test_command(Channel::ReliableSequence(ChannelGroup(1), ChannelSequence(2)), 3);
		let cmd_2_a = create_test_command(Channel::ReliableSequence(ChannelGroup(2), ChannelSequence(0)), 4);
		let cmd_2_b = create_test_command(Channel::ReliableSequence(ChannelGroup(2), ChannelSequence(1)), 5);
		let cmd_2_c = create_test_command(Channel::ReliableSequence(ChannelGroup(2), ChannelSequence(2)), 6);

		assert(1, &mut in_commands, &[cmd_1_a.clone()], &[cmd_1_a]);
		assert(6, &mut in_commands, &[cmd_2_b.clone()], &[]);
		assert(5, &mut in_commands, &[cmd_1_c.clone()], &[]);
		assert(2, &mut in_commands, &[cmd_2_a.clone()], &[cmd_2_a, cmd_2_b]);
		assert(3, &mut in_commands, &[cmd_1_b.clone()], &[cmd_1_b, cmd_1_c]);
		assert(4, &mut in_commands, &[cmd_2_c.clone()], &[cmd_2_c]);
	}

	fn assert(
		frame_id: FrameId,
		in_commands: &mut InCommandsCollector,
		commands: &[CommandWithChannel],
		expect: &[CommandWithChannel],
	) {
		let mut frame = Frame::new(frame_id);
		for command in commands {
			frame.commands.push(command.clone()).unwrap();
		}
		in_commands.collect(frame);
		assert_eq!(in_commands.get_ready_commands(), expect);
	}
	fn create_test_command(channel: Channel, content: i64) -> CommandWithChannel {
		create_test_object_command(channel, 0, content)
	}

	fn create_test_object_command(channel: Channel, object_id: u32, content: i64) -> CommandWithChannel {
		CommandWithChannel {
			channel,
			both_direction_command: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
				object_id: GameObjectId::new(object_id, GameObjectOwner::Room),
				field_id: 0,
				value: content,
			})),
		}
	}
}
