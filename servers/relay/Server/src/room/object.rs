use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::constants::{FieldIdType, GameObjectTemplateType};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::UserPublicKey;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template: GameObjectTemplateType,
	pub access_groups: AccessGroups,
	///
	/// Объект полностью создан
	///
	pub created: bool,
	pub longs: HashMap<FieldIdType, i64, FnvBuildHasher>,
	pub floats: HashMap<FieldIdType, f64, FnvBuildHasher>,
	pub structures: HashMap<FieldIdType, Vec<u8>, FnvBuildHasher>,
	pub compare_and_set_owners: HashMap<FieldIdType, UserPublicKey, FnvBuildHasher>,
}

impl GameObject {
	pub fn new(id: GameObjectId) -> Self {
		Self {
			id,
			template: 0,
			access_groups: Default::default(),
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
			compare_and_set_owners: Default::default(),
		}
	}

	pub fn collect_create_commands(&self, commands: &mut Vec<S2CCommand>) {
		commands.push(S2CCommand::Create(CreateGameObjectCommand {
			object_id: self.id.clone(),
			template: self.template.clone(),
			access_groups: self.access_groups.clone(),
		}));

		self.structures_to_commands(commands);
		self.longs_to_commands(commands);
		self.floats_to_commands(commands);

		if self.created {
			commands.push(S2CCommand::Created(CreatedGameObjectCommand { object_id: self.id.clone() }));
		}
	}
}

#[cfg(test)]
mod tests {

	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;

	use crate::room::object::GameObject;

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub fn should_collect_command() {
		let id = GameObjectId::new(1, ObjectOwner::Root);
		let mut object = GameObject::new(id.clone());
		object.template = 55;
		object.access_groups = AccessGroups(63);
		object.created = true;
		object.longs.insert(1, 100);
		object.floats.insert(2, 200.200);
		object.structures.insert(1, vec![1, 2, 3]);

		let mut commands = Vec::new();
		object.collect_create_commands(&mut commands);

		assert!(matches!(commands.remove(0),
			S2CCommand::Create(c) if c.object_id==id && c.template == object.template && c.access_groups == object.access_groups));
		assert!(matches!(commands.remove(0), S2CCommand::SetStruct(c) if c.object_id==id && c.field_id == 1 && c.structure.to_vec() == vec![1,2,3]));
		assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id==id && c.field_id == 1 && c.value == 100));
		assert!(matches!(commands.remove(0), S2CCommand::SetFloat(c) if c.object_id==id && c.field_id == 2 && c.value == 200.200));
		assert!(matches!(commands.remove(0), S2CCommand::Created(c) if c.object_id==id));
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, ObjectOwner::Root);
		let mut object = GameObject::new(id.clone());
		object.longs.insert(1, 100);

		let mut commands = Vec::new();
		object.collect_create_commands(&mut commands);
		assert!(matches!(commands.remove(0), S2CCommand::Create(_)));
		assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id==id && c.field_id == 1 && c.value == 100));
		assert_eq!(commands.len(), 0)
	}
}
