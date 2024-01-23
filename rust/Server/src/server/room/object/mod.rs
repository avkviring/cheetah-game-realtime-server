use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

use crate::server::room::config::object::GameObjectConfig;
use cheetah_common::commands::s2c::S2CCommand;
use cheetah_common::commands::types::create::{CreateGameObject, GameObjectCreated};
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::{GameObjectId, GameObjectTemplateId};
use fields::Fields;

use crate::server::room::object::fields::structure::Structure;
use crate::server::room::object::fields::vec::Items;

pub mod fields;

pub const MAX_FIELD_COUNT: usize = 64;

pub type S2CCommandsCollector = Vec<S2CCommand>;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template_id: GameObjectTemplateId,
	pub config: Arc<GameObjectConfig>,
	pub access_groups: AccessGroups,
	pub created: bool,
	pub double_fields: Fields<f64>,
	pub long_fields: Fields<i64>,
	pub structure_fields: Fields<Structure>,
	pub structures_fields: Fields<Items>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GameObjectError {
	#[error("Field count overflow in game object {0:?} with config {1:?}")]
	FieldCountOverflow(GameObjectId, GameObjectTemplateId),
}

impl GameObject {
	#[must_use]
	pub fn new(id: GameObjectId, template_id: GameObjectTemplateId, access_groups: AccessGroups, config: Arc<GameObjectConfig>, created: bool) -> Self {
		Self {
			id,
			template_id,
			config,
			access_groups,
			created,
			double_fields: Default::default(),
			long_fields: Default::default(),
			structure_fields: Default::default(),
			structures_fields: Default::default(),
		}
	}

	pub fn collect_create_commands(&mut self, out_commands: &mut S2CCommandsCollector) {
		out_commands.push(S2CCommand::Create(CreateGameObject {
			object_id: self.id,
			template: self.template_id,
			access_groups: self.access_groups,
		}));
		self.fields_to_commands(out_commands);
		if self.created {
			out_commands.push(S2CCommand::Created(GameObjectCreated { object_id: self.id }));
		}
	}

	fn fields_to_commands(&mut self, commands: &mut S2CCommandsCollector) {
		self.long_fields.collect_commands(commands, self.id);
		self.double_fields.collect_commands(commands, self.id);
		self.structure_fields.collect_commands(commands, self.id);
		self.structures_fields.collect_commands(commands, self.id);
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::s2c::S2CCommand;
	use cheetah_common::commands::types::create::GameObjectCreated;
	use cheetah_common::commands::types::float::DoubleField;
	use cheetah_common::commands::types::long::LongField;
	use cheetah_common::commands::types::structure::BinaryField;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::server::room::object::{GameObject, S2CCommandsCollector};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub(crate) fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 55, AccessGroups(63), Default::default(), true);
		object.long_fields.set(1, 100);
		object.double_fields.set(2, 200.200);
		object.structure_fields.set(1, [1, 2, 3].as_ref().into());
		object.structures_fields.set(1, [[1, 2, 3].as_ref().into(), [4, 5, 6].as_ref().into()].into_iter().collect());

		let mut commands = S2CCommandsCollector::new();
		object.collect_create_commands(&mut commands);

		assert!(matches!(
			&commands[0],
			S2CCommand::Create(c)
			if c.object_id==id
			&& c.template == object.template_id
			&& c.access_groups == object.access_groups
		));

		assert_eq!(
			commands[1],
			S2CCommand::SetLong(LongField {
				object_id: id,
				field_id: 1,
				value: 100,
			})
		);

		assert_eq!(
			commands[2],
			S2CCommand::SetDouble(DoubleField {
				object_id: id,
				field_id: 2,
				value: 200.200,
			})
		);

		assert_eq!(
			commands[3],
			S2CCommand::SetStructure(
				BinaryField {
					object_id: id,
					field_id: 1,
					value: [1, 2, 3].as_ref().into(),
				}
				.into()
			)
		);
		assert_eq!(
			commands[4],
			S2CCommand::AddItem(
				BinaryField {
					object_id: id,
					field_id: 1,
					value: [1, 2, 3].as_ref().into(),
				}
				.into()
			)
		);

		assert_eq!(
			commands[5],
			S2CCommand::AddItem(
				BinaryField {
					object_id: id,
					field_id: 1,
					value: [4, 5, 6].as_ref().into(),
				}
				.into()
			)
		);

		assert_eq!(commands[6], S2CCommand::Created(GameObjectCreated { object_id: id }));
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub(crate) fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 0, Default::default(), Default::default(), false);
		object.long_fields.set(1, 100);

		let mut commands = S2CCommandsCollector::new();
		object.collect_create_commands(&mut commands);
		assert_eq!(commands.len(), 2);
		assert!(matches!(&commands[0], S2CCommand::Create(_)));
		assert!(matches!(&commands[1],
			S2CCommand::SetLong(c)
			if c.object_id==id && c.field_id== 1 && c.value == 100));
	}

	#[test]
	pub(crate) fn should_update_structure() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), Default::default(), false);
		object.structure_fields.set(1, Buffer::from([1, 2, 3].as_ref()));
		object.structure_fields.set(1, Buffer::from([4, 5, 6, 7].as_ref()));

		let s: &Buffer = object.structure_fields.get(1).unwrap();
		assert_eq!(s.as_slice(), [4, 5, 6, 7]);
	}
}
