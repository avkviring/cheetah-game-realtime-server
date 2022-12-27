use std::fmt::{Debug, Formatter};

use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use cheetah_common::commands::types::delete::DeleteGameObjectCommand;
use cheetah_common::commands::types::event::EventCommand;
use cheetah_common::commands::types::field::DeleteFieldCommand;
use cheetah_common::commands::types::float::SetDoubleCommand;
use cheetah_common::commands::types::long::SetLongCommand;
use cheetah_common::commands::types::member::{MemberConnected, MemberDisconnected};
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::commands::CommandTypeId;

use crate::clients::registry::ClientId;
use crate::ffi::execute_with_client;

pub mod event;
pub mod field;
pub mod float_value;
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
					CommandTypeId::SetStructure => self.command.set_structure.eq(&other.command.set_structure),
					CommandTypeId::SendEvent => self.command.event.eq(&other.command.event),
					CommandTypeId::DeleteObject => self.command.delete.eq(&other.command.delete),
					CommandTypeId::DeleteField => self.command.delete_field.eq(&other.command.delete_field),
					CommandTypeId::MemberConnected => self.command.member_connect.eq(&other.command.member_connect),
					CommandTypeId::MemberDisconnected => self.command.member_disconnect.eq(&other.command.member_disconnect),
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
	pub create: CreateGameObjectCommand,
	pub created: GameObjectCreatedS2CCommand,
	pub set_long: SetLongCommand,
	pub set_double: SetDoubleCommand,
	pub set_structure: SetStructureCommand,
	pub event: EventCommand,
	pub delete: DeleteGameObjectCommand,
	pub delete_field: DeleteFieldCommand,
	pub member_connect: MemberConnected,
	pub member_disconnect: MemberDisconnected,
}
