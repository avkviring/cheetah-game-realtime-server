use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::commands::c2s::C2SCommand;
use crate::commands::context::CommandContextError;
use crate::commands::guarantees::{ChannelSequence, ReliabilityGuarantees, ReliabilityGuaranteesChannel};
use crate::commands::s2c::S2CCommand;
use crate::room::object::GameObjectId;

pub mod c2s;
pub mod codec;
pub mod context;
pub mod guarantees;
pub mod s2c;
pub mod types;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CommandWithReliabilityGuarantees {
	pub reliability_guarantees: ReliabilityGuaranteesChannel,
	pub command: BothDirectionCommand,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CommandWithChannelType {
	pub command: BothDirectionCommand,
	pub channel_type: ReliabilityGuarantees,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum BothDirectionCommand {
	S2C(S2CCommand),
	C2S(C2SCommand),
}

impl ChannelSequence {
	pub const FIRST: ChannelSequence = ChannelSequence(0);

	#[must_use]
	pub fn next(&self) -> Self {
		ChannelSequence(self.0 + 1)
	}
}

impl BothDirectionCommand {
	#[must_use]
	pub fn get_object_id(&self) -> Option<GameObjectId> {
		match &self {
			BothDirectionCommand::S2C(command) => command.get_object_id(),
			BothDirectionCommand::C2S(command) => command.get_object_id(),
		}
	}
}

///
/// Идентификатор типа команды
///
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, FromPrimitive, ToPrimitive, Hash, Serialize, Deserialize)]
pub enum CommandTypeId {
	CreateGameObject = 0,
	CreatedGameObject,
	SetLong,
	IncrementLong,
	SetDouble,
	IncrementDouble,
	SetStructure,
	SendEvent,
	TargetEvent,
	DeleteObject,
	AttachToRoom,
	DetachFromRoom,
	DeleteField,
	MemberConnected,
	MemberDisconnected,
	AddItem,
}

#[derive(Error, Debug)]
pub enum CommandDecodeError {
	#[error("Unknown type {0:?}.")]
	UnknownTypeId(CommandTypeId),
	#[error("IO error {0}")]
	Io(#[from] std::io::Error),
	#[error("CommandContext error {0}")]
	CommandContext(#[from] CommandContextError),
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::commands::context::CommandContextError;
	use crate::commands::s2c::S2CCommand;
	use crate::commands::types::create::{CreateGameObject, GameObjectCreated};
	use crate::commands::types::float::DoubleField;
	use crate::commands::types::long::LongField;
	use crate::commands::types::member::MemberConnected;
	use crate::commands::types::structure::BinaryField;
	use crate::commands::CommandTypeId;
	use crate::room::access::AccessGroups;
	use crate::room::buffer::Buffer;
	use crate::room::field::FieldId;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	#[test]
	fn should_decode_encode_create() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(
			&S2CCommand::Create(CreateGameObject {
				object_id,
				template: 3,
				access_groups: AccessGroups(5),
			}),
			CommandTypeId::CreateGameObject,
			Some(object_id),
			None,
		);
	}

	#[test]
	fn should_decode_encode_created() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(&S2CCommand::Created(GameObjectCreated { object_id }), CommandTypeId::CreatedGameObject, Some(object_id), None);
	}

	#[test]
	fn should_decode_encode_set_long() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::SetLong(LongField { object_id, field_id, value: 100 }),
			CommandTypeId::SetLong,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_double() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::SetDouble(DoubleField { object_id, field_id, value: 3.15 }),
			CommandTypeId::SetDouble,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_set_structure() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::SetStructure(
				BinaryField {
					object_id,
					field_id,
					value: Buffer::from([1, 2, 3, 4].as_ref()),
				}
				.into(),
			),
			CommandTypeId::SetStructure,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_event() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::Event(
				BinaryField {
					object_id,
					field_id,
					value: Buffer::from(vec![1, 2, 3, 4].as_slice()),
				}
				.into(),
			),
			CommandTypeId::SendEvent,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_ring_buffer_command() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		let field_id = 77;
		check(
			&S2CCommand::AddItem(
				BinaryField {
					object_id,
					field_id,
					value: Buffer::from(vec![1, 2, 3, 4].as_slice()),
				}
				.into(),
			),
			CommandTypeId::AddItem,
			Some(object_id),
			Some(field_id),
		);
	}

	#[test]
	fn should_decode_encode_delete() {
		let object_id = GameObjectId::new(100, GameObjectOwner::Room);
		check(&S2CCommand::Delete(object_id), CommandTypeId::DeleteObject, Some(object_id), None);
	}

	#[test]
	fn should_decode_encode_member_connected() {
		check(&S2CCommand::MemberConnected(MemberConnected { member_id: 100 }), CommandTypeId::MemberConnected, None, None);
	}

	fn check(expected: &S2CCommand, command_type_id: CommandTypeId, object_id: Option<GameObjectId>, field_id: Option<FieldId>) {
		let object_id = object_id.ok_or(CommandContextError::ContextNotContainsObjectId);
		let field_id = field_id.ok_or(CommandContextError::ContextNotContainsFieldId);
		let mut buffer = [0_u8; 100];
		let mut cursor = Cursor::new(buffer.as_mut());
		expected.encode(&mut cursor).unwrap();
		let write_position = cursor.position();
		let mut read_cursor = Cursor::<&[u8]>::new(&buffer);
		let actual = S2CCommand::decode(&command_type_id, object_id, field_id, &mut read_cursor).unwrap();

		assert_eq!(write_position, read_cursor.position());
		assert_eq!(expected, &actual);
	}
}
