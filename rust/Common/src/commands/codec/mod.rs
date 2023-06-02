pub mod decoder;
pub mod encoder;

///
/// Преобразование списка команд в бинарное представление и обратно
/// docs/adr/matches/0005-relay-frame-format.md
///
#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use cheetah_protocol::frame::FRAME_BODY_CAPACITY;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::codec::decoder::decode_commands;
	use crate::commands::codec::encoder::encode_commands;
	use crate::commands::guarantees::{ChannelGroup, ChannelSequence, ReliabilityGuaranteesChannel};
	use crate::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use crate::commands::types::float::SetDoubleCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::commands::{BothDirectionCommand, CommandWithReliabilityGuarantees};
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn test_c2s() {
		let commands = vec![
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::SetDouble(SetDoubleCommand {
					object_id: Default::default(),
					field_id: 10,
					value: 1.5,
				})),
			},
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableSequence(ChannelGroup(11), ChannelSequence(12)),
				command: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
					object_id: GameObjectId::new(13, GameObjectOwner::Member(14)),
					field_id: 15,
					value: 16,
				})),
			},
		]
		.into();

		check(true, commands);
	}

	#[test]
	fn test_s2s() {
		let commands = vec![
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
					command: S2CCommand::SetDouble(SetDoubleCommand {
						object_id: Default::default(),
						field_id: 10,
						value: 1.5,
					}),
					creator: 55,
				}),
			},
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableSequence(ChannelGroup(11), ChannelSequence(12)),
				command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
					command: S2CCommand::SetLong(SetLongCommand {
						object_id: Default::default(),
						field_id: 5,
						value: 1,
					}),
					creator: 57,
				}),
			},
		];
		check(false, commands);
	}

	#[test]
	fn stress_test() {
		let mut commands = vec![];
		for i in 0..1000 {
			let command = CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
					command: S2CCommand::SetDouble(SetDoubleCommand {
						object_id: Default::default(),
						field_id: i,
						value: 1.5,
					}),
					creator: 55,
				}),
			};

			commands.push(command);
		}
		check(false, commands);
	}

	#[allow(clippy::cast_possible_truncation)]
	fn check(server_side: bool, original_commands: Vec<CommandWithReliabilityGuarantees>) {
		let mut cloned_original_commands: VecDeque<_> = original_commands.clone().into();
		let mut i = 0;
		while !cloned_original_commands.is_empty() {
			let mut buffer = [0; FRAME_BODY_CAPACITY];
			let (size, _) = encode_commands(&mut cloned_original_commands, &mut buffer);
			for command in decode_commands(server_side, &buffer[0..size]).unwrap() {
				assert_eq!(original_commands[i], command);
				i += 1;
			}
		}
	}
}
