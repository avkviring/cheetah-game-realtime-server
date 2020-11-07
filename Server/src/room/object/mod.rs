use std::cell::RefCell;
use std::rc::Rc;
use std::task::Context;

use cheetah_relay_common::commands::command::S2CCommandUnion;
use cheetah_relay_common::commands::hash::UserPublicKey;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::room::command::CommandContext;
use crate::room::object::server_object_id::ServerGameObjectId;

pub mod server_object_id;


///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone)]
pub struct GameObject {
	pub id: ServerGameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}


