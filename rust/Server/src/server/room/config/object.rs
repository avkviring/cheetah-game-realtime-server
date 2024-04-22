use std::collections::HashMap;

use cheetah_game_realtime_protocol::RoomMemberId;
use fnv::{FnvBuildHasher, FnvHashMap};
use serde::{Deserialize, Serialize};

use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};
use cheetah_common::room::owner::GameObjectOwner;

use crate::server::room::object::GameObject;
use crate::server::room::Room;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameObjectConfig {
	pub items_config: FnvHashMap<FieldId, ItemConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemConfig {
	pub capacity: usize,
}

static DEFAULT_ITEM_CONFIG: ItemConfig = ItemConfig { capacity: 50 };

impl Default for &ItemConfig {
	fn default() -> Self {
		&DEFAULT_ITEM_CONFIG
	}
}

impl Default for ItemConfig {
	fn default() -> Self {
		DEFAULT_ITEM_CONFIG.clone()
	}
}

impl GameObjectConfig {
	pub fn get_items_config(&self, field_id: &FieldId) -> &ItemConfig {
		self.items_config.get(field_id).unwrap_or_default()
	}
}

impl GameObjectCreateParams {
	#[must_use]
	pub fn to_root_game_object(&self, room: &Room) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Room), room)
	}

	#[must_use]
	pub fn create_member_game_object(&self, member_id: RoomMemberId, room: &Room) -> GameObject {
		self.to_game_object(GameObjectId::new(self.id, GameObjectOwner::Member(member_id)), room)
	}

	#[must_use]
	pub fn to_game_object(&self, id: GameObjectId, room: &Room) -> GameObject {
		assert_ne!(id.id, 0, "0 is forbidden for game object id");

		let config = room.get_object_config(&self.template);
		let mut object = GameObject::new(id, self.template, self.groups, config, true);

		self.longs.iter().for_each(|(&k, v)| object.long_fields.set(k, *v));
		self.doubles.iter().for_each(|(&k, v)| object.double_fields.set(k, *v));
		self.structures.iter().for_each(|(&k, v)| object.structure_fields.set(k, Box::new(*v)));

		object
	}
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GameObjectCreateParams {
	pub id: u32,
	pub template: GameObjectTemplateId,
	pub groups: AccessGroups,
	pub longs: HashMap<FieldId, i64, FnvBuildHasher>,
	pub doubles: HashMap<FieldId, f64, FnvBuildHasher>,
	pub structures: HashMap<FieldId, Buffer, FnvBuildHasher>,
}

#[cfg(test)]
mod tests {
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::config::object::GameObjectCreateParams;
	use crate::server::room::Room;

	#[test]
	#[should_panic]
	fn should_panic_if_object_id_is_0() {
		let config_object = GameObjectCreateParams {
			id: 0,
			template: 200,
			groups: Default::default(),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		let room = Room::new(Default::default(), Default::default());
		let _object = config_object.to_root_game_object(&room);
	}

	#[test]
	#[allow(clippy::float_cmp)]
	fn should_convert_game_object() {
		let mut config_object = GameObjectCreateParams {
			id: 100,
			template: 200,
			groups: Default::default(),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};

		config_object.longs.insert(0, 100);
		config_object.doubles.insert(1, 105.105);
		config_object.structures.insert(2, [1].as_ref().into());

		let room = Room::new(Default::default(), Default::default());
		let object = config_object.clone().to_root_game_object(&room);
		assert_eq!(config_object.id, object.id.id);
		assert!(matches!(object.id.get_owner(), GameObjectOwner::Room));
		assert_eq!(config_object.template, object.template_id);
		assert_eq!(config_object.groups, object.access_groups);

		let config_value = config_object.longs.get(&0).unwrap();
		let object_value = object.long_fields.get(0).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value = config_object.doubles.get(&1).unwrap();
		let object_value = object.double_fields.get(1).unwrap();
		assert_eq!(*config_value, *object_value);

		let config_value = config_object.structures.get(&2).unwrap();
		let object_value = object.structure_fields.get(2).unwrap();
		assert_eq!(*config_value, **object_value);
	}
}
