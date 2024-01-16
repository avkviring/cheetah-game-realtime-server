use std::fmt::{Debug, Formatter};

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::create::{CreateGameObject, GameObjectCreated};
use cheetah_common::commands::types::field::DeleteField;
use cheetah_common::commands::types::float::DoubleField;
use cheetah_common::commands::types::long::LongField;
use cheetah_common::commands::types::member::{MemberConnected, MemberDisconnected};
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::object::GameObjectId;

use crate::clients::registry::ClientId;
use crate::ffi::execute_with_client;

pub mod event;
pub mod field;
pub mod float_value;
pub mod items;
pub mod long_value;
pub mod object;
pub mod room;
pub mod structure;

fn send_command(client_id: ClientId, command: C2SCommand) -> u8 {
	execute_with_client(client_id, |client| Ok(client.send(command)?))
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct S2CCommandFFI {
	pub command_type: CommandTypeId,
	pub command: S2CommandUnionFFI,
}

impl PartialEq for S2CCommandFFI {
	fn eq(&self, other: &Self) -> bool {
		unsafe {
			self.command_type == other.command_type
				&& match self.command_type {
					CommandTypeId::CreateGameObject => self.command.create.eq(&other.command.create),
					CommandTypeId::CreatedGameObject => self.command.created.eq(&other.command.created),
					CommandTypeId::SetLong => self.command.set_long.eq(&other.command.set_long),
					CommandTypeId::SetDouble => self.command.set_double.eq(&other.command.set_double),
					CommandTypeId::SetStructure => self.command.buffer_field.eq(&other.command.buffer_field),
					CommandTypeId::SendEvent => self.command.buffer_field.eq(&other.command.buffer_field),
					CommandTypeId::DeleteObject => self.command.game_object_id.eq(&other.command.game_object_id),
					CommandTypeId::DeleteField => self.command.delete_field.eq(&other.command.delete_field),
					CommandTypeId::MemberConnected => self.command.member_connect.eq(&other.command.member_connect),
					CommandTypeId::MemberDisconnected => self.command.member_disconnect.eq(&other.command.member_disconnect),
					CommandTypeId::AddItem => self.command.buffer_field.eq(&other.command.buffer_field),
					_ => false,
				}
		}
	}
}

impl Debug for S2CCommandFFI {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("S2CCommandFFI").field("command_type", &self.command_type).finish()
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union S2CommandUnionFFI {
	pub empty: (),
	pub create: CreateGameObject,
	pub created: GameObjectCreated,
	pub set_long: LongField,
	pub set_double: DoubleField,
	pub buffer_field: BinaryField,
	pub game_object_id: GameObjectId,
	pub delete_field: DeleteField,
	pub member_connect: MemberConnected,
	pub member_disconnect: MemberDisconnected,
}
