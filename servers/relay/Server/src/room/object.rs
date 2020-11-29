use serde::{Deserialize, Serialize};

use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	pub fields: GameObjectFields,
}
