use fnv::FnvBuildHasher;
use indexmap::IndexMap;

use cheetah_matches_relay_common::commands::command::C2SCommand;
use cheetah_matches_relay_common::protocol::frame::applications::ApplicationCommands;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::{RoomId, UserId};

use crate::room::object::GameObject;

///
/// Сервис визуализации потока сетевых команд для отладки
/// adr/matches/0002-relay-debug-commands-flow-in-unity.md
///
///
pub mod filter;
pub mod parser;

pub struct CommandTracerSessions {}

impl CommandTracerSessions {
	pub(crate) fn on_c2s(&self, room_id: &RoomId, objects: &IndexMap<GameObjectId, GameObject, FnvBuildHasher>, user: &UserId, command: &C2SCommand) {
		//todo!()
	}
	pub(crate) fn on_s2c(
		&self,
		room_id: &RoomId,
		objects: &IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
		user: &UserId,
		commands: &ApplicationCommands,
	) {
		//todo!()
	}
}

impl Default for CommandTracerSessions {
	fn default() -> Self {
		Self {}
	}
}

impl CommandTracerSessions {}
