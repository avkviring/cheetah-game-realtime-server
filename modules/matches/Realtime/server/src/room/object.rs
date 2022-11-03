use thiserror::Error;

use cheetah_matches_realtime_common::commands::field::{Field, FieldId};
use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
use cheetah_matches_realtime_common::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use cheetah_matches_realtime_common::commands::{field::ToFieldType, FieldType, FieldValue};
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::access::AccessGroups;
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::RoomMemberId;

const TYPE_COUNT: usize = 3;
pub const MAX_FIELD_COUNT: usize = 64;
type FieldIndex = heapless::FnvIndexMap<(FieldId, FieldType), FieldValue, { MAX_FIELD_COUNT * TYPE_COUNT }>;
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

#[derive(Error, Debug, PartialEq)]
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

	pub fn delete_field(&mut self, field_id: FieldId, field_type: FieldType) {
		self.fields.remove(&(field_id, field_type));
	}

	pub(crate) fn fields(&self) -> &FieldIndex {
		&self.fields
	}

	pub fn get_field<'a, T: 'a>(&'a self, field_id: FieldId) -> Option<&'a T>
	where
		FieldValue: AsRef<T>,
		T: ToFieldType,
	{
		let field_type = T::to_field_type();
		self.fields.get(&(field_id, field_type)).map(|v| v.as_ref())
	}

	pub fn get_field_wrapped(&self, field_id: FieldId, field_type: FieldType) -> Option<&FieldValue> {
		self.fields.get(&(field_id, field_type))
	}

	pub fn set_field<T>(&mut self, field_id: FieldId, value: T) -> Result<(), GameObjectError>
	where
		FieldValue: From<T>,
		T: ToFieldType,
	{
		let field_value = value.into();
		self.set_field_wrapped(field_id, field_value)
	}

	pub fn set_field_wrapped(&mut self, field_id: FieldId, value: FieldValue) -> Result<(), GameObjectError> {
		let field_type = value.field_type();
		self.fields
			.insert((field_id, field_type), value)
			.map(|_| ())
			.map_err(|_| GameObjectError::FieldCountOverflow(self.id.to_owned(), self.template_id))
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
		if self.do_collect_create_commands(commands).is_err() {
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
				command: S2CCommand::Created(GameObjectCreatedS2CCommand { object_id: self.id.clone() }),
			})?;
		}
		Ok(())
	}

	fn fields_to_commands(&self, commands: &mut CreateCommandsCollector) -> Result<(), S2CCommandWithFieldInfo> {
		for (&(field_id, field_type), v) in self.fields() {
			let command = S2CCommandWithFieldInfo {
				field: Option::Some(Field { id: field_id, field_type }),
				command: S2CCommand::new_set_command(v.to_owned(), self.id.to_owned(), field_id),
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

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::field::Field;
	use cheetah_matches_realtime_common::commands::s2c::S2CCommand;
	use cheetah_matches_realtime_common::commands::FieldType;
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;

	use crate::room::object::{CreateCommandsCollector, FieldValue, GameObject, S2CCommandWithFieldInfo};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 55, AccessGroups(63), true);
		object.set_field(1, 100).unwrap();
		object.set_field(2, 200.200).unwrap();
		object
			.fields
			.insert((1, FieldType::Structure), FieldValue::Structure(vec![1, 2, 3]))
			.unwrap();

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands);

		assert!(matches!(
			&commands[0],
			S2CCommandWithFieldInfo { field: None, command:S2CCommand::Create(c) }
			if c.object_id==id
			&& c.template == object.template_id
			&& c.access_groups == object.access_groups
		));

		assert!(matches!(
			&commands[1],
			S2CCommandWithFieldInfo {
				field: Some(Field { id: 1, field_type: FieldType::Long }),
				command: S2CCommand::SetField(c)
			}
			if c.object_id==id && c.field_id == 1 && c.value == 100.into()
		));

		assert!({
			if let S2CCommandWithFieldInfo {
				field: Some(Field {
					id: 2,
					field_type: FieldType::Double,
				}),
				command: S2CCommand::SetField(c),
			} = &commands[2]
			{
				let v: f64 = c.to_owned().value.into();
				let values_close = (v - 200.200).abs() < 0.0001;
				c.object_id == id && c.field_id == 2 && values_close
			} else {
				false
			}
		});

		assert!(matches!(
			&commands[3],
			S2CCommandWithFieldInfo {
				field: Some(Field { id: 1, field_type: FieldType::Structure }),
				command: S2CCommand::SetField(c)
			}
			if c.object_id==id && c.field_id == 1 && c.value == vec![1,2,3].into()
		));

		assert!(matches!(
			&commands[4],
			S2CCommandWithFieldInfo {
				 field: None,  command: S2CCommand::Created(c)
			}
			if c.object_id == id));
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id.clone(), 0, Default::default(), false);
		object.set_field(1, 100).unwrap();

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
			S2CCommandWithFieldInfo { field: Some(Field { id: 1, field_type: FieldType::Long }), command:S2CCommand::SetField(c)}
			if c.object_id==id && c.field_id== 1 && c.value == 100.into()));
	}

	#[test]
	pub fn should_update_structure() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);
		object.set_field(1, [1, 2, 3].as_ref()).unwrap();
		object.set_field(1, [4, 5, 6, 7].as_ref()).unwrap();

		let s: &Vec<u8> = object.get_field(1).unwrap();
		assert_eq!(*s, [4, 5, 6, 7])
	}
}
