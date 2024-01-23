use crate::server::room::config::object::GameObjectConfig;
use crate::server::room::config::object::GameObjectCreateParams;
use cheetah_common::room::object::GameObjectTemplateId;
use fnv::FnvHashMap;

///
/// Шаблон для создания комнаты
///
#[derive(Debug, Default, Clone)]
pub struct RoomCreateParams {
	pub name: String,
	pub objects: Vec<GameObjectCreateParams>,
	pub configs: FnvHashMap<GameObjectTemplateId, GameObjectConfig>,
}
