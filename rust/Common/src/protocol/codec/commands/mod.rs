pub mod context;
///
/// Преобразование списка команд в бинарное представление и обратно
/// docs/adr/matches/0005-relay-frame-format.md
///
pub mod decoder;
pub mod encoder;
mod header;

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use byteorder::WriteBytesExt;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use crate::commands::types::float::SetDoubleCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::protocol::codec::commands::context::CommandContext;
	use crate::protocol::codec::commands::decoder::decode_commands;
	use crate::protocol::codec::commands::encoder::encode_command;
	use crate::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, ChannelSequence, CommandWithReliabilityGuarantees};
	use crate::protocol::frame::channel::ReliabilityGuaranteesChannel;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn test_c2s() {
		let commands = vec![
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				commands: BothDirectionCommand::C2S(C2SCommand::SetDouble(SetDoubleCommand {
					object_id: Default::default(),
					field_id: 10,
					value: 1.5,
				})),
			},
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableSequence(ChannelGroup(11), ChannelSequence(12)),
				commands: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
					object_id: GameObjectId::new(13, GameObjectOwner::Member(14)),
					field_id: 15,
					value: 16,
				})),
			},
		];

		check(true, &commands);
	}

	#[test]
	fn test_s2s() {
		let commands = vec![
			CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				commands: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
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
				commands: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
					command: S2CCommand::SetLong(SetLongCommand {
						object_id: Default::default(),
						field_id: 5,
						value: 1,
					}),
					creator: 57,
				}),
			},
		];
		check(false, &commands);
	}

	#[allow(clippy::cast_possible_truncation)]
	fn check(from_client: bool, commands: &[CommandWithReliabilityGuarantees]) {
		let mut buffer = [0_u8; 64];
		let mut cursor = Cursor::new(buffer.as_mut());
		let mut context = CommandContext::default();
		cursor.write_u8(commands.len() as u8).unwrap();
		for command in commands {
			encode_command(&mut context, command, &mut cursor).unwrap();
		}
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let mut readed = Default::default();
		decode_commands(from_client, &mut read_cursor, &mut readed).unwrap();
		assert_eq!(write_position, read_cursor.position());
		assert_eq!(commands, readed);
	}
}
