use std::collections::HashMap;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::RoomMemberId;

///
/// Игровой объект - логическая группировка игровых данных
///
#[derive(Debug, Clone)]
pub struct GameObject {
	pub id: GameObjectId,
	pub template_id: GameObjectTemplateId,
	pub access_groups: AccessGroups,
	///
	/// Объект полностью создан
	///
	pub created: bool,
	longs: heapless::FnvIndexMap<FieldId, i64, MAX_FIELD_COUNT>,
	floats: heapless::FnvIndexMap<FieldId, f64, MAX_FIELD_COUNT>,
	pub structures: HashMap<FieldId, Vec<u8>, FnvBuildHasher>,
	pub compare_and_set_owners: heapless::FnvIndexMap<FieldId, RoomMemberId, MAX_FIELD_COUNT>,
}
pub const MAX_FIELD_COUNT: usize = 64;
pub type CreateCommandsCollector = heapless::Vec<S2CommandWithFieldInfo, 255>;

impl GameObject {
	pub fn new(id: GameObjectId, template_id: GameObjectTemplateId, access_groups: AccessGroups, created: bool) -> Self {
		Self {
			id,
			template_id,
			access_groups,
			created,
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
			compare_and_set_owners: Default::default(),
		}
	}

	pub fn get_longs(&self) -> &heapless::FnvIndexMap<FieldId, i64, MAX_FIELD_COUNT> {
		&self.longs
	}
	pub fn get_long(&self, field_id: &FieldId) -> Option<&i64> {
		self.longs.get(field_id)
	}
	pub fn set_long(&mut self, field_id: FieldId, value: i64) {
		if self.longs.insert(field_id, value).is_err() {
			log::error!("Long count fields overflow")
		}
	}

	pub fn get_floats(&self) -> &heapless::FnvIndexMap<FieldId, f64, MAX_FIELD_COUNT> {
		&self.floats
	}
	pub fn get_float(&self, field_id: &FieldId) -> Option<&f64> {
		self.floats.get(field_id)
	}
	pub fn set_float(&mut self, field_id: FieldId, value: f64) {
		if self.floats.insert(field_id, value).is_err() {
			log::error!("Long count fields overflow")
		}
	}

	pub fn collect_create_commands(&self, commands: &mut CreateCommandsCollector) {
		match self.do_collect_create_commands(commands) {
			Ok(_) => {}
			Err(_) => {
				log::error!("Collect create commands overflow {:?}", self);
			}
		}
	}

	fn do_collect_create_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CommandWithFieldInfo> {
		commands.push(S2CommandWithFieldInfo {
			field: Option::None,
			command: S2CCommand::Create(CreateGameObjectCommand {
				object_id: self.id.clone(),
				template: self.template_id,
				access_groups: self.access_groups,
			}),
		})?;

		self.structures_to_commands(commands)?;
		self.longs_to_commands(commands)?;
		self.floats_to_commands(commands)?;

		if self.created {
			commands.push(S2CommandWithFieldInfo {
				field: None,
				command: S2CCommand::Created(CreatedGameObjectCommand {
					object_id: self.id.clone(),
				}),
			})?;
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct S2CommandWithFieldInfo {
	pub field: Option<Field>,
	pub command: S2CCommand,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Field {
	pub id: FieldId,
	pub field_type: FieldType,
}

#[cfg(test)]
mod tests {
	use cheetah_matches_relay_common::commands::s2c::S2CCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::room::object::{CreateCommandsCollector, Field, GameObject, S2CommandWithFieldInfo};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 55, AccessGroups(63), true);
		object.set_long(1, 100);
		object.set_float(2, 200.200);
		object.structures.insert(1, vec![1, 2, 3]);

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands);

		assert!(matches!(&commands[0],
			S2CommandWithFieldInfo { field: None, command:S2CCommand::Create(c) } if c.object_id==id && c.template == object.template_id && c.access_groups == object.access_groups));

		assert!(matches!(&commands[1],
			S2CommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Structure }), command:S2CCommand::SetStructure(c) }
			if c.object_id==id && c.field_id == 1 && c.structure.to_vec() == vec![1,2,3]));

		assert!(matches!(&commands[2],
			S2CommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Long }), command: S2CCommand::SetLong(c)}
			if c.object_id==id && c.field_id == 1 && c.value == 100));

		assert!(matches!(&commands[3],
			S2CommandWithFieldInfo { field: Some(Field { id: 2, field_type: FieldType::Double }),  command: S2CCommand::SetDouble(c)}
			if c.object_id==id && c.field_id == 2 && (c.value - 200.200).abs() < 0.0001));

		assert!(
			matches!(&commands[4],S2CommandWithFieldInfo { field: None,  command: S2CCommand::Created(c)} if c.object_id==id)
		);
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 0, Default::default(), false);
		object.set_long(1, 100);

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands);
		assert_eq!(commands.len(), 2);
		assert!(matches!(
			&commands[0],
			S2CommandWithFieldInfo {
				field: None,
				command: S2CCommand::Create(_)
			}
		));
		assert!(matches!(&commands[1],
			S2CommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Long }), command:S2CCommand::SetLong(c)}
			if c.object_id==id && c.field_id== 1 && c.value == 100));
	}
}
