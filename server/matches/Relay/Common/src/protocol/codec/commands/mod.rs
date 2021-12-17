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
	use std::collections::VecDeque;
	use std::io::Cursor;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use crate::commands::types::float::SetDoubleCommand;
	use crate::commands::types::long::SetLongCommand;
	
	use crate::protocol::codec::commands::decoder::decode_commands;
	use crate::protocol::codec::commands::encoder::encode_commands;
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use crate::protocol::frame::channel::Channel;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn test_c2s() {
		let mut commands = VecDeque::new();
		commands.push_back(CommandWithChannel {
			channel: Channel::ReliableUnordered,
			command: BothDirectionCommand::C2S(C2SCommand::SetDouble(SetDoubleCommand {
				object_id: Default::default(),
				field_id: 10,
				value: 1.5,
			})),
		});
		commands.push_back(CommandWithChannel {
			channel: Channel::ReliableSequenceByGroup(11, 12),
			command: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
				object_id: GameObjectId::new(13, GameObjectOwner::User(14)),
				field_id: 15,
				value: 16,
			})),
		});
		check(true, commands);
	}

	#[test]
	fn test_s2s() {
		let mut commands = VecDeque::new();
		commands.push_back(CommandWithChannel {
			channel: Channel::ReliableUnordered,
			command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
				command: S2CCommand::SetDouble(SetDoubleCommand {
					object_id: Default::default(),
					field_id: 10,
					value: 1.5,
				}),
				creator: 55,
			}),
		});
		commands.push_back(CommandWithChannel {
			channel: Channel::ReliableSequenceByGroup(11, 12),
			command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: Default::default(),
					field_id: 5,
					value: 1,
				}),
				creator: 57,
			}),
		});
		check(false, commands);
	}

	fn check(from_client: bool, commands: VecDeque<CommandWithChannel>) {
		let mut buffer = [0_u8; 64];
		let mut cursor = Cursor::new(buffer.as_mut());
		encode_commands(&commands, &mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let mut readed = VecDeque::new();
		decode_commands(from_client, &mut read_cursor, &mut readed).unwrap();
		assert_eq!(write_position, read_cursor.position());
		assert_eq!(commands, readed);
	}
}
