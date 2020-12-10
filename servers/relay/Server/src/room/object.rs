use std::collections::HashMap;

use fnv::FnvBuildHasher;
use serde::{Deserialize, Serialize};

use cheetah_relay_common::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::UserPublicKey;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template: u16,
	pub access_groups: AccessGroups,
	///
	/// Объект полностью создан
	///
	pub created: bool,
	pub longs: HashMap<FieldID, i64, FnvBuildHasher>,
	pub floats: HashMap<FieldID, f64, FnvBuildHasher>,
	pub structures: HashMap<FieldID, Vec<u8>, FnvBuildHasher>,
	pub compare_and_set_owners: HashMap<FieldID, UserPublicKey, FnvBuildHasher>,
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
