use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use fnv::FnvBuildHasher;

use crate::protocol::frame::applications::{ChannelGroup, ChannelSequence, CommandWithChannel};
use crate::protocol::frame::channel::Channel;
use crate::protocol::frame::{Frame, FrameId};
use crate::room::object::GameObjectId;

///
/// Коллектор входящих команд
/// - поддержка мультиплексирования
///
#[derive(Default, Debug)]
pub struct InCommandsCollector {
	ordered: HashMap<ChannelKey, FrameId, FnvBuildHasher>,
	sequence_commands: HashMap<ChannelKey, BinaryHeap<SequenceApplicationCommand>, FnvBuildHasher>,
	sequence_last: HashMap<ChannelKey, ChannelSequence, FnvBuildHasher>,
	commands: VecDeque<CommandWithChannel>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum ChannelKey {
	Group(ChannelGroup),
	ClientGameObjectId(GameObjectId),
}

impl InCommandsCollector {
	pub fn get_commands(&mut self) -> &mut VecDeque<CommandWithChannel> {
		&mut self.commands
	}

	pub fn collect(&mut self, frame: Frame) {
		let frame_id = frame.frame_id;
		let commands = frame.reliable.into_iter().chain(frame.unreliable.into_iter());

		commands.into_iter().for_each(|c| {
			match c.channel {
				Channel::ReliableUnordered | Channel::UnreliableUnordered => self.commands.push_front(c),

				Channel::ReliableOrderedByObject | Channel::UnreliableOrderedByObject => {
					if let Some(object_id) = c.command.get_object_id() {
						self.process_ordered(ChannelKey::ClientGameObjectId(object_id.clone()), frame_id, c);
					}
				}
				Channel::ReliableOrderedByGroup(group) | Channel::UnreliableOrderedByGroup(group) => {
					self.process_ordered(ChannelKey::Group(group), frame_id, c);
				}

				Channel::ReliableSequenceByObject(sequence) => {
					if let Some(object_id) = c.command.get_object_id().cloned() {
						self.process_sequence(ChannelKey::ClientGameObjectId(object_id), sequence, c);
					}
				}

				Channel::ReliableSequenceByGroup(channel_id, sequence) => {
					self.process_sequence(ChannelKey::Group(channel_id), sequence, c)
				}
			};
		});
	}

	fn process_sequence(&mut self, channel_key: ChannelKey, sequence: u32, command: CommandWithChannel) {
		let mut last = *self.sequence_last.get(&channel_key).unwrap_or(&0);
		if sequence == 0 || sequence == last + 1 {
			last = sequence;
			self.commands.push_front(command);

			match self.sequence_commands.get_mut(&channel_key) {
				None => {}
				Some(buffer) => {
					while let Option::Some(peek) = buffer.peek() {
						let sequence = peek.sequence;
						if sequence == last + 1 {
							self.commands.push_front(buffer.pop().unwrap().command);
							last = sequence;
						} else {
							break;
						}
					}
				}
			}

			self.sequence_last.insert(channel_key, last);
		} else {
			let buffer = self.sequence_commands.entry(channel_key).or_insert_with(BinaryHeap::default);
			buffer.push(SequenceApplicationCommand { sequence, command });
		}
	}

	fn process_ordered(&mut self, channel_key: ChannelKey, frame_id: FrameId, command: CommandWithChannel) {
		match self.ordered.get(&channel_key) {
			None => {
				self.ordered.insert(channel_key, frame_id);
				self.commands.push_front(command);
			}
			Some(processed_frame_id) if frame_id >= *processed_frame_id => {
				self.ordered.insert(channel_key, frame_id);
				self.commands.push_front(command);
			}
			_ => {}
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
		self.sequence.cmp(&other.sequence).reverse()
	}

	fn max(self, other: Self) -> Self
	where
		Self: Sized,
	{
		if self.sequence > other.sequence {
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
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::protocol::frame::Frame;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	pub fn test_unordered() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_command(Channel::ReliableUnordered, 1);
		let command_2 = create_test_command(Channel::ReliableUnordered, 2);

		let mut frame1 = Frame::new(2);
		frame1.unreliable.push(command_1.clone()).unwrap();

		let mut frame2 = Frame::new(2);
		frame2.unreliable.push(command_2.clone()).unwrap();

		in_commands.collect(frame2);
		in_commands.collect(frame1);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
	}

	#[test]
	pub fn test_group_ordered() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_command(Channel::ReliableOrderedByGroup(1), 1);
		let command_2 = create_test_command(Channel::ReliableOrderedByGroup(1), 2);
		let command_3 = create_test_command(Channel::ReliableOrderedByGroup(1), 3);

		let mut frame1 = Frame::new(1);
		frame1.reliable.push(command_1.clone()).unwrap();

		let mut frame2 = Frame::new(2);
		frame2.reliable.push(command_2).unwrap();

		let mut frame3 = Frame::new(3);
		frame3.reliable.push(command_3.clone()).unwrap();

		in_commands.collect(frame1);
		in_commands.collect(frame3);
		in_commands.collect(frame2);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_3);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_ordered_when_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_command(Channel::ReliableOrderedByGroup(1), 1);
		let command_2 = create_test_command(Channel::ReliableOrderedByGroup(2), 2);

		let mut frame1 = Frame::new(1);
		frame1.reliable.push(command_1.clone()).unwrap();

		let mut frame2 = Frame::new(2);
		frame2.reliable.push(command_2.clone()).unwrap();

		in_commands.collect(frame2);
		in_commands.collect(frame1);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
	}

	#[test]
	pub fn test_object_ordered() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_object_command(Channel::ReliableOrderedByObject, 1, 1);
		let command_2 = create_test_object_command(Channel::ReliableOrderedByObject, 1, 2);
		let command_3 = create_test_object_command(Channel::ReliableOrderedByObject, 1, 3);

		let mut frame1 = Frame::new(1);
		frame1.reliable.push(command_1.clone()).unwrap();

		let mut frame2 = Frame::new(2);
		frame2.reliable.push(command_2).unwrap();

		let mut frame3 = Frame::new(3);
		frame3.reliable.push(command_3.clone()).unwrap();

		in_commands.collect(frame1);
		in_commands.collect(frame3);
		in_commands.collect(frame2);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_3);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_ordered_with_different_object() {
		let mut in_commands = InCommandsCollector::default();

		let command_1_a = create_test_object_command(Channel::ReliableOrderedByObject, 1, 1);
		let command_1_b = create_test_object_command(Channel::ReliableOrderedByObject, 1, 2);
		let command_1_c = create_test_object_command(Channel::ReliableOrderedByObject, 1, 3);

		let command_2_a = create_test_object_command(Channel::ReliableOrderedByObject, 2, 1);
		let command_2_b = create_test_object_command(Channel::ReliableOrderedByObject, 2, 2);
		let command_2_c = create_test_object_command(Channel::ReliableOrderedByObject, 2, 3);

		let mut frame1 = Frame::new(1);
		frame1.reliable.push(command_1_a.clone()).unwrap();
		frame1.reliable.push(command_2_a.clone()).unwrap();

		let mut frame2 = Frame::new(2);
		frame2.reliable.push(command_1_b).unwrap();
		frame2.reliable.push(command_2_b).unwrap();

		let mut frame3 = Frame::new(3);
		frame3.reliable.push(command_1_c.clone()).unwrap();
		frame3.reliable.push(command_2_c.clone()).unwrap();

		in_commands.collect(frame1);
		in_commands.collect(frame3);
		in_commands.collect(frame2);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_c);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_c);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_sequence() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_command(Channel::ReliableSequenceByGroup(1, 1), 1);
		let command_2 = create_test_command(Channel::ReliableSequenceByGroup(1, 2), 2);
		let command_3 = create_test_command(Channel::ReliableSequenceByGroup(1, 3), 3);
		let command_4 = create_test_command(Channel::ReliableSequenceByGroup(1, 4), 4);
		let command_5 = create_test_command(Channel::ReliableSequenceByGroup(1, 5), 5);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_3.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_5.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_4.clone()).unwrap();
		in_commands.collect(frame);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_3);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_4);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_5);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_sequence_with_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let command_1_a = create_test_command(Channel::ReliableSequenceByGroup(1, 1), 1);
		let command_1_b = create_test_command(Channel::ReliableSequenceByGroup(1, 2), 2);
		let command_1_c = create_test_command(Channel::ReliableSequenceByGroup(1, 3), 3);

		let command_2_a = create_test_command(Channel::ReliableSequenceByGroup(2, 1), 4);
		let command_2_b = create_test_command(Channel::ReliableSequenceByGroup(2, 2), 5);
		let command_2_c = create_test_command(Channel::ReliableSequenceByGroup(2, 3), 6);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_a.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_b.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_c.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_a.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_b.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_c.clone()).unwrap();
		in_commands.collect(frame);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_b);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_b);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_c);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_c);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_sequence() {
		let mut in_commands = InCommandsCollector::default();

		let command_1 = create_test_object_command(Channel::ReliableSequenceByObject(0), 1, 1);
		let command_2 = create_test_object_command(Channel::ReliableSequenceByObject(1), 1, 2);
		let command_3 = create_test_object_command(Channel::ReliableSequenceByObject(2), 1, 3);
		let command_4 = create_test_object_command(Channel::ReliableSequenceByObject(3), 1, 4);
		let command_5 = create_test_object_command(Channel::ReliableSequenceByObject(4), 1, 5);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_3.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_5.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_4.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2.clone()).unwrap();
		in_commands.collect(frame);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_3);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_4);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_5);

		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_sequence_with_different_objects() {
		let mut in_commands = InCommandsCollector::default();

		let command_1_a = create_test_object_command(Channel::ReliableSequenceByObject(1), 1, 1);
		let command_1_b = create_test_object_command(Channel::ReliableSequenceByObject(2), 1, 2);
		let command_1_c = create_test_object_command(Channel::ReliableSequenceByObject(3), 1, 3);

		let command_2_a = create_test_object_command(Channel::ReliableSequenceByObject(1), 2, 1);
		let command_2_b = create_test_object_command(Channel::ReliableSequenceByObject(2), 2, 2);
		let command_2_c = create_test_object_command(Channel::ReliableSequenceByObject(3), 2, 3);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_a.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_b.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_c.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_a.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_1_b.clone()).unwrap();
		in_commands.collect(frame);

		let mut frame = Frame::new(0);
		frame.reliable.push(command_2_c.clone()).unwrap();
		in_commands.collect(frame);

		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_a);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_b);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_b);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_1_c);
		assert_eq!(in_commands.get_commands().pop_back().unwrap(), command_2_c);
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	fn create_test_command(channel: Channel, content: i64) -> CommandWithChannel {
		create_test_object_command(channel, 0, content)
	}

	fn create_test_object_command(channel: Channel, object_id: u32, content: i64) -> CommandWithChannel {
		CommandWithChannel {
			channel,
			command: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
				object_id: GameObjectId::new(object_id, GameObjectOwner::Room),
				field_id: 0,
				value: content,
			})),
		}
	}
}
