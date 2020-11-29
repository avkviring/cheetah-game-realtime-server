use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use fnv::FnvBuildHasher;

use crate::protocol::frame::applications::{ApplicationCommandChannel, ApplicationCommandDescription, ChannelGroupId, ChannelSequence};
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
	commands: VecDeque<ApplicationCommandDescription>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum ChannelKey {
	Group(ChannelGroupId),
	ClientGameObjectId(GameObjectId),
}

impl InCommandsCollector {
	pub fn get_commands(&mut self) -> &mut VecDeque<ApplicationCommandDescription> {
		&mut self.commands
	}

	pub fn collect(&mut self, frame: Frame) {
		let frame_id = frame.header.frame_id.clone();
		let commands = frame.commands.reliable.into_iter().chain(frame.commands.unreliable.into_iter());

		commands.into_iter().for_each(|c| {
			match c.channel {
				ApplicationCommandChannel::ReliableUnordered | ApplicationCommandChannel::UnreliableUnordered => self.commands.push_front(c),

				ApplicationCommandChannel::ReliableOrderedByObject | ApplicationCommandChannel::UnreliableOrderedByObject => {
					if let Some(object_id) = c.command.get_object_id() {
						self.process_ordered(ChannelKey::ClientGameObjectId(object_id.clone()), frame_id, c);
					}
				}
				ApplicationCommandChannel::ReliableOrderedByGroup(group) | ApplicationCommandChannel::UnreliableOrderedByGroup(group) => {
					self.process_ordered(ChannelKey::Group(group), frame_id, c);
				}

				ApplicationCommandChannel::ReliableSequenceByObject(sequence) => {
					if let Some(object_id) = c.command.get_object_id().cloned() {
						self.process_sequence(ChannelKey::ClientGameObjectId(object_id.clone()), sequence, c);
					}
				}

				ApplicationCommandChannel::ReliableSequenceByGroup(channel_id, sequence) => {
					self.process_sequence(ChannelKey::Group(channel_id), sequence, c)
				}
			};
		});
	}

	fn process_sequence(&mut self, channel_key: ChannelKey, sequence: u32, command: ApplicationCommandDescription) {
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
			let buffer = self.sequence_commands.entry(channel_key).or_insert_with(|| BinaryHeap::default());
			buffer.push(SequenceApplicationCommand { sequence, command });
		}
	}

	fn process_ordered(&mut self, channel_key: ChannelKey, frame_id: FrameId, command: ApplicationCommandDescription) {
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
	command: ApplicationCommandDescription,
}

impl PartialEq for SequenceApplicationCommand {
	fn eq(&self, other: &Self) -> bool {
		self.sequence.eq(&other.sequence)
	}
}

impl PartialOrd for SequenceApplicationCommand {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Option::Some(self.cmp(&other))
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
	use crate::protocol::commands::input::InCommandsCollector;
	use crate::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
	use crate::protocol::frame::Frame;
	use crate::room::object::GameObjectId;
	use crate::room::owner::ObjectOwner;

	#[test]
	pub fn test_unordered() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();

		in_commands.collect(Frame::new(2).add_command(ApplicationCommandChannel::ReliableUnordered, content_2.clone()));
		in_commands.collect(Frame::new(1).add_command(ApplicationCommandChannel::ReliableUnordered, content_1.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content == content_2));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content == content_1));
	}

	#[test]
	pub fn test_group_ordered() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();
		let content_3 = "command_3".to_string();

		in_commands.collect(Frame::new(1).add_command(ApplicationCommandChannel::ReliableOrderedByGroup(1), content_1.clone()));
		in_commands.collect(Frame::new(3).add_command(ApplicationCommandChannel::ReliableOrderedByGroup(1), content_3.clone()));
		in_commands.collect(Frame::new(2).add_command(ApplicationCommandChannel::ReliableOrderedByGroup(1), content_2.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content) if content==content_1));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_3));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_ordered_when_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();

		in_commands.collect(Frame::new(2).add_command(ApplicationCommandChannel::ReliableOrderedByGroup(1), content_2.clone()));
		in_commands.collect(Frame::new(1).add_command(ApplicationCommandChannel::ReliableOrderedByGroup(2), content_1.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content == "command_2"));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content == "command_1"));
	}

	#[test]
	pub fn test_object_ordered() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();
		let content_3 = "command_3".to_string();

		in_commands.collect(Frame::new(1).add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_1.clone()));
		in_commands.collect(Frame::new(3).add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_3.clone()));
		in_commands.collect(Frame::new(2).add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_2.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_3));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_ordered_with_different_object() {
		let mut in_commands = InCommandsCollector::default();

		let content_1_a = "command_1_a".to_string();
		let content_1_b = "command_1_b".to_string();
		let content_1_c = "command_1_c".to_string();

		let content_2_a = "command_2_a".to_string();
		let content_2_b = "command_2_b".to_string();
		let content_2_c = "command_2_c".to_string();

		in_commands.collect(
			Frame::new(1)
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_1_a.clone())
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 2, content_2_a.clone()),
		);

		in_commands.collect(
			Frame::new(3)
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_1_c.clone())
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 2, content_2_c.clone()),
		);

		// этот фрейм не должен быть учтен
		in_commands.collect(
			Frame::new(2)
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 1, content_1_b.clone())
				.add_object_command(ApplicationCommandChannel::ReliableOrderedByObject, 2, content_2_b.clone()),
		);

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1_c));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2_c));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_sequence() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();
		let content_3 = "command_3".to_string();
		let content_4 = "command_4".to_string();
		let content_5 = "command_5".to_string();

		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 0), content_1.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 2), content_3.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 4), content_5.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 3), content_4.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 1), content_2.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content)if content==content_1));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content) if content==content_2));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content) if content==content_3));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content) if content==content_4));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command,ApplicationCommand::TestSimple(content) if content==content_5));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_group_sequence_with_different_group() {
		let mut in_commands = InCommandsCollector::default();

		let content_1_a = "command_1_a".to_string();
		let content_1_b = "command_1_b".to_string();
		let content_1_c = "command_1_c".to_string();

		let content_2_a = "command_2_a".to_string();
		let content_2_b = "command_2_b".to_string();
		let content_2_c = "command_2_c".to_string();

		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 1), content_1_a.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(2, 2), content_2_b.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 3), content_1_c.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(2, 1), content_2_a.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(1, 2), content_1_b.clone()));
		in_commands.collect(Frame::new(0).add_command(ApplicationCommandChannel::ReliableSequenceByGroup(2, 3), content_2_c.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_1_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_2_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_2_b));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_1_b));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_1_c));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestSimple(content) if content==content_2_c));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_sequence() {
		let mut in_commands = InCommandsCollector::default();

		let content_1 = "command_1".to_string();
		let content_2 = "command_2".to_string();
		let content_3 = "command_3".to_string();
		let content_4 = "command_4".to_string();
		let content_5 = "command_5".to_string();

		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(0), 1, content_1.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(2), 1, content_3.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(4), 1, content_5.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(3), 1, content_4.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(1), 1, content_2.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_3));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_4));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_5));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	#[test]
	pub fn test_object_sequence_with_different_objects() {
		let mut in_commands = InCommandsCollector::default();

		let content_1_a = "command_1_a".to_string();
		let content_1_b = "command_1_b".to_string();
		let content_1_c = "command_1_c".to_string();

		let content_2_a = "command_2_a".to_string();
		let content_2_b = "command_2_b".to_string();
		let content_2_c = "command_2_c".to_string();

		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(1), 1, content_1_a.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(2), 2, content_2_b.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(3), 1, content_1_c.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(1), 2, content_2_a.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(2), 1, content_1_b.clone()));
		in_commands.collect(Frame::new(0).add_object_command(ApplicationCommandChannel::ReliableSequenceByObject(3), 2, content_2_c.clone()));

		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2_a));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2_b));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1_b));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_1_c));
		assert!(matches!(in_commands.get_commands().pop_back().unwrap().command, ApplicationCommand::TestObject(_,content) if content==content_2_c));
		assert!(matches!(in_commands.get_commands().pop_back(), Option::None));
	}

	impl Frame {
		fn add_command(mut self, channel: ApplicationCommandChannel, content: String) -> Self {
			self.commands.reliable.push(ApplicationCommandDescription {
				channel,
				command: ApplicationCommand::TestSimple(content),
			});
			self
		}

		fn add_object_command(mut self, channel: ApplicationCommandChannel, object_id: u32, content: String) -> Self {
			let command_description = ApplicationCommandDescription {
				channel,
				command: ApplicationCommand::TestObject(GameObjectId::new(object_id, ObjectOwner::Root), content),
			};
			self.commands.reliable.push(command_description);
			self
		}
	}
}
