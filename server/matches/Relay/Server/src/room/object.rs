use thiserror::Error;

use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::create::{CreateGameObjectCommand, S2CreatedGameObjectCommand};
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::RoomMemberId;

use crate::room::field::FieldValue;

const TYPE_COUNT: usize = 3;
pub const MAX_FIELD_COUNT: usize = 64;
type FieldIndex = heapless::FnvIndexMap<FieldId, FieldValue, { MAX_FIELD_COUNT * TYPE_COUNT }>;
pub type CreateCommandsCollector = heapless::Vec<S2CCommandWithFieldInfo, 255>;

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
	fields: FieldIndex,
	compare_and_set_owners: heapless::FnvIndexMap<FieldId, RoomMemberId, MAX_FIELD_COUNT>,
}

#[derive(Error, Debug)]
pub enum GameObjectError {
	#[error("Field count overflow in game object {:?} with template {:?}", .0, .1)]
	FieldCountOverflow(GameObjectId, GameObjectTemplateId),
}

impl GameObject {
	pub fn new(id: GameObjectId, template_id: GameObjectTemplateId, access_groups: AccessGroups, created: bool) -> Self {
		Self {
			id,
			template_id,
			access_groups,
			created,
			fields: Default::default(),
			compare_and_set_owners: Default::default(),
		}
	}

	pub fn delete_field(&mut self, field_id: FieldId) {
		self.fields.remove(&field_id);
	}

	pub(crate) fn fields(&self) -> &FieldIndex {
		&self.fields
	}

	pub fn field(&self, field_id: &FieldId) -> Option<&FieldValue> {
		self.fields.get(field_id)
	}

	pub fn set_field(&mut self, field_id: FieldId, value: &FieldValue) -> Result<(), GameObjectError> {
		self.fields
			.insert(field_id, value.to_owned())
			.map(|_| ())
			.map_err(|_| GameObjectError::FieldCountOverflow(self.id.to_owned(), self.template_id))
	}

	pub fn get_long(&self, field_id: &FieldId) -> Option<&i64> {
		let f = self.field(field_id)?;
		if let FieldValue::Long(v) = f {
			Some(v)
		} else {
			panic!("The requested field was not of type long")
		}
	}

	pub fn set_long(&mut self, field_id: FieldId, value: i64) -> Result<(), GameObjectError> {
		self.set_field(field_id, &FieldValue::Long(value))
	}

	pub fn get_double(&self, field_id: &FieldId) -> Option<&f64> {
		let f = self.field(field_id)?;
		if let FieldValue::Double(v) = f {
			Some(v)
		} else {
			panic!("The requested field was not of type double")
		}
	}

	pub fn set_double(&mut self, field_id: FieldId, value: f64) -> Result<(), GameObjectError> {
		self.set_field(field_id, &FieldValue::Double(value))
	}

	pub fn get_structure(&self, field_id: &FieldId) -> Option<&Vec<u8>> {
		let f = self.field(field_id)?;
		if let FieldValue::Structure(s) = f {
			Some(s)
		} else {
			panic!("The requested field was not of type structure");
		}
	}

	pub fn set_structure(&mut self, field_id: FieldId, structure: &[u8]) -> Result<(), GameObjectError> {
		match self.fields.get_mut(&field_id) {
			Some(field_value) => {
				if let FieldValue::Structure(vec) = field_value {
					vec.clear();
					vec.extend_from_slice(structure);
				} else {
					panic!("The provided value was of type structure");
				}
				Ok(())
			}
			None => self
				.fields
				.insert(field_id, FieldValue::Structure(structure.into()))
				.map(|_| ())
				.map_err(|_| GameObjectError::FieldCountOverflow(self.id.clone(), self.template_id)),
		}
	}

	pub fn get_compare_and_set_owners(&self) -> &heapless::FnvIndexMap<FieldId, RoomMemberId, MAX_FIELD_COUNT> {
		&self.compare_and_set_owners
	}

	pub fn get_compare_and_set_owner(&self, field_id: &FieldId) -> Option<&RoomMemberId> {
		self.compare_and_set_owners.get(field_id)
	}

	pub fn set_compare_and_set_owner(&mut self, field_id: FieldId, value: RoomMemberId) -> Result<(), GameObjectError> {
		self.compare_and_set_owners
			.insert(field_id, value)
			.map(|_| ())
			.map_err(|_| GameObjectError::FieldCountOverflow(self.id.clone(), self.template_id))
	}

	pub fn collect_create_commands(&self, commands: &mut CreateCommandsCollector) {
		if let Err(_) = self.do_collect_create_commands(commands) {
			tracing::error!("Collect create commands overflow {:?}", self);
		}
	}

	fn do_collect_create_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CCommandWithFieldInfo> {
		commands.push(S2CCommandWithFieldInfo {
			field: Option::None,
			command: S2CCommand::Create(CreateGameObjectCommand {
				object_id: self.id.clone(),
				template: self.template_id,
				access_groups: self.access_groups,
			}),
		})?;

		self.fields_to_commands(commands)?;

		if self.created {
			commands.push(S2CCommandWithFieldInfo {
				field: None,
				command: S2CCommand::Created(S2CreatedGameObjectCommand {
					object_id: self.id.clone(),
				}),
			})?;
		}
		Ok(())
	}

	fn fields_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CCommandWithFieldInfo> {
		for (field_id, v) in self.fields() {
			let command = S2CCommandWithFieldInfo {
				field: Option::Some(Field {
					id: *field_id,
					field_type: v.get_type(),
				}),
				command: v.s2c_set_command(self.id.clone(), *field_id),
			};
			commands.push(command)?;
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct S2CCommandWithFieldInfo {
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

	use crate::room::object::{CreateCommandsCollector, Field, FieldValue, GameObject, S2CCommandWithFieldInfo};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 55, AccessGroups(63), true);
		object.set_long(1, 100).unwrap();
		object.set_double(2, 200.200).unwrap();
		object.fields.insert(3, FieldValue::Structure(vec![1, 2, 3])).unwrap();

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands);

		assert!(matches!(&commands[0],
			S2CCommandWithFieldInfo { field: None, command:S2CCommand::Create(c) } if c.object_id==id && c.template == object.template_id && c.access_groups == object.access_groups));

		assert!(matches!(&commands[1],
			S2CCommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Long }), command: S2CCommand::SetLong(c)}
			if c.object_id==id && c.field_id == 1 && c.value == 100));

		assert!(matches!(&commands[2],
			S2CCommandWithFieldInfo { field: Some(Field { id: 2, field_type: FieldType::Double }),  command: S2CCommand::SetDouble(c)}
			if c.object_id==id && c.field_id == 2 && (c.value - 200.200).abs() < 0.0001));

		assert!(matches!(&commands[3],
			S2CCommandWithFieldInfo { field: Some(Field { id: 3, field_type: FieldType::Structure }), command:S2CCommand::SetStructure(c) }
			if c.object_id==id && c.field_id == 3 && c.value.as_slice().to_vec() == vec![1,2,3]));

		assert!(
			matches!(&commands[4], S2CCommandWithFieldInfo { field: None,  command: S2CCommand::Created(c)} if c.object_id==id)
		);
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 0, Default::default(), false);
		object.set_long(1, 100).unwrap();

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands);
		assert_eq!(commands.len(), 2);
		assert!(matches!(
			&commands[0],
			S2CCommandWithFieldInfo {
				field: None,
				command: S2CCommand::Create(_)
			}
		));
		assert!(matches!(&commands[1],
			S2CCommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Long }), command:S2CCommand::SetLong(c)}
			if c.object_id==id && c.field_id== 1 && c.value == 100));
	}

	#[test]
	pub fn should_update_structure() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);
		object.set_structure(1, &[1, 2, 3]).unwrap();
		object.set_structure(1, &[4, 5, 6, 7]).unwrap();
		assert_eq!(*object.get_structure(&1).unwrap(), [4, 5, 6, 7])
	}
}
