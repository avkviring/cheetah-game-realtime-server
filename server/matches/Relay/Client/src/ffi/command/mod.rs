use cheetah_matches_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::room::UserId;

use crate::ffi::{execute_with_client, GameObjectIdFFI};

pub mod event;
pub mod float_value;
pub mod long_value;
pub mod object;
pub mod room;
pub mod structure;

fn send_command(command: C2SCommand) -> bool {
	execute_with_client(|client| client.send(command)).is_ok()
}

#[repr(C)]
pub struct S2CMetaCommandInformationFFI {
	///
	/// Идентификатор клиента
	///
	pub user_id: UserId,

	///
	/// Условное время создание команды на клиенте
	///
	pub timestamp: u64,

	///
	/// Объект - источник команды
	///
	pub source_object: GameObjectIdFFI,
}

impl S2CMetaCommandInformationFFI {
	pub fn stub() -> Self {
		Self {
			user_id: 15,
			timestamp: 25,
			source_object: GameObjectIdFFI {
				id: 3,
				room_owner: false,
				user_id: 5,
			},
		}
	}
}

impl From<&S2CMetaCommandInformation> for S2CMetaCommandInformationFFI {
	fn from(source: &S2CMetaCommandInformation) -> Self {
		Self {
			user_id: source.user_id,
			timestamp: source.timestamp,
			source_object: match &source.source_object {
				None => GameObjectIdFFI::empty(),
				Some(object) => From::from(object),
			},
		}
	}
}
