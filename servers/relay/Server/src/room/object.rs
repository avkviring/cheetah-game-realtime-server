use serde::{Deserialize, Serialize};

use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::UserPublicKey;
use fnv::FnvBuildHasher;
use std::collections::HashMap;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub longs: HashMap<FieldID, i64, FnvBuildHasher>,
	pub floats: HashMap<FieldID, f64, FnvBuildHasher>,
	pub structures: HashMap<FieldID, Vec<u8>, FnvBuildHasher>,
	pub compare_and_set_owners: HashMap<FieldID, UserPublicKey, FnvBuildHasher>,
}
