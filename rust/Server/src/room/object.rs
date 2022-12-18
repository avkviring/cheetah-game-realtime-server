use thiserror::Error;

use cheetah_common::commands::field::{Field, FieldId};
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use cheetah_common::commands::{field::ToFieldType, FieldType, FieldValue};
use cheetah_common::constants::GameObjectTemplateId;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

pub const MAX_FIELD_COUNT: usize = 64;

type FieldIndex = heapless::FnvIndexMap<(FieldId, FieldType), FieldValue, { MAX_FIELD_COUNT * 4 }>;
pub type CreateCommandsCollector = heapless::Vec<S2CCommandWithMeta, 255>;

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

#[derive(Error, Debug, PartialEq, Eq)]
pub enum GameObjectError {
	#[error("Field count overflow in game object {0:?} with template {1:?}")]
	FieldCountOverflow(GameObjectId, GameObjectTemplateId),
}

impl GameObject {
	#[must_use]
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

	pub fn get_field<T>(&self, field_id: FieldId) -> Option<&T>
	where
		FieldValue: AsRef<T>,
		T: ToFieldType,
	{
		let field_type = T::to_field_type();
		self.fields.get(&(field_id, field_type)).map(AsRef::as_ref)
	}

	#[must_use]
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

	#[allow(clippy::map_err_ignore)]
	pub fn set_field_wrapped(&mut self, field_id: FieldId, value: FieldValue) -> Result<(), GameObjectError> {
		let field_type = value.field_type();
		self.fields
			.insert((field_id, field_type), value)
			.map(|_| ())
			.map_err(|_| GameObjectError::FieldCountOverflow(self.id, self.template_id))
	}

	#[must_use]
	pub fn get_compare_and_set_owners(&self) -> &heapless::FnvIndexMap<FieldId, RoomMemberId, MAX_FIELD_COUNT> {
		&self.compare_and_set_owners
	}

	#[must_use]
	pub fn get_compare_and_set_owner(&self, field_id: &FieldId) -> Option<&RoomMemberId> {
		self.compare_and_set_owners.get(field_id)
	}

	#[allow(clippy::map_err_ignore)]
	pub fn set_compare_and_set_owner(&mut self, field_id: FieldId, value: RoomMemberId) -> Result<(), GameObjectError> {
		self.compare_and_set_owners
			.insert(field_id, value)
			.map(|_| ())
			.map_err(|_| GameObjectError::FieldCountOverflow(self.id, self.template_id))
	}

	pub fn collect_create_commands(&self, commands: &mut CreateCommandsCollector, member_id: RoomMemberId) {
		if self.do_collect_create_commands(commands, member_id).is_err() {
			tracing::error!("Collect create commands overflow {:?}", self);
		}
	}

	fn do_collect_create_commands(&self, commands: &mut CreateCommandsCollector, member_id: RoomMemberId) -> Result<(), S2CCommandWithMeta> {
		commands.push(S2CCommandWithMeta {
			field: None,
			creator: member_id,
			command: S2CCommand::Create(CreateGameObjectCommand {
				object_id: self.id,
				template: self.template_id,
				access_groups: self.access_groups,
			}),
		})?;

		self.fields_to_commands(commands, member_id)?;

		if self.created {
			commands.push(S2CCommandWithMeta {
				field: None,
				creator: member_id,
				command: S2CCommand::Created(GameObjectCreatedS2CCommand { object_id: self.id }),
			})?;
		}
		Ok(())
	}

	fn fields_to_commands(&self, commands: &mut CreateCommandsCollector, member_id: RoomMemberId) -> Result<(), S2CCommandWithMeta> {
		for (&(field_id, field_type), v) in self.fields() {
			let command = S2CCommandWithMeta {
				field: Some(Field { id: field_id, field_type }),
				creator: member_id,
				command: S2CCommand::new_set_command(v.clone(), self.id, field_id),
			};
			commands.push(command)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::binary_value::BinaryValue;
	use cheetah_common::commands::field::Field;
	use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
	use cheetah_common::commands::FieldType;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::object::{CreateCommandsCollector, FieldValue, GameObject};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub(crate) fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 55, AccessGroups(63), true);
		object.set_field(1, 100).unwrap();
		object.set_field(2, 200.200).unwrap();
		object
			.fields
			.insert((1, FieldType::Structure), FieldValue::Structure([1, 2, 3].as_ref().into()))
			.unwrap();

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands, u16::MAX);

		assert!(matches!(
			&commands[0],
			S2CCommandWithMeta { field: None, creator: u16::MAX, command:S2CCommand::Create(c) }
			if c.object_id==id
			&& c.template == object.template_id
			&& c.access_groups == object.access_groups
		));

		assert!(matches!(
			&commands[1],
			S2CCommandWithMeta {
				field: Some(Field { id: 1, field_type: FieldType::Long }),
				creator: u16::MAX,
				command: S2CCommand::SetField(c)
			}
			if c.object_id==id && c.field_id == 1 && c.value == 100.into()
		));

		assert!({
			if let S2CCommandWithMeta {
				field: Some(Field {
					id: 2,
					field_type: FieldType::Double,
				}),
				creator: u16::MAX,
				command: S2CCommand::SetField(c),
			} = &commands[2]
			{
				let v: f64 = c.clone().value.into();
				let values_close = (v - 200.200).abs() < 0.0001;
				c.object_id == id && c.field_id == 2 && values_close
			} else {
				false
			}
		});

		assert!(matches!(
			&commands[3],
			S2CCommandWithMeta {
				field: Some(Field { id: 1, field_type: FieldType::Structure }),
				creator: u16::MAX,
				command: S2CCommand::SetField(c)
			}
			if c.object_id==id && c.field_id == 1 && AsRef::<BinaryValue>::as_ref(&c.value).as_slice()==[1,2,3].as_ref()
		));

		assert!(matches!(
			&commands[4],
			S2CCommandWithMeta {
				 field: None, creator: u16::MAX, command: S2CCommand::Created(c)
			}
			if c.object_id == id));
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub(crate) fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 0, Default::default(), false);
		object.set_field(1, 100).unwrap();

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands, u16::MAX);
		assert_eq!(commands.len(), 2);
		assert!(matches!(
			&commands[0],
			S2CCommandWithMeta {
				field: None,
				creator: u16::MAX,
				command: S2CCommand::Create(_)
			}
		));
		assert!(matches!(&commands[1],
			S2CCommandWithMeta {
				field: Some(Field { id: 1, field_type: FieldType::Long }),
				creator: u16::MAX,
				command:S2CCommand::SetField(c)
			}
			if c.object_id==id && c.field_id== 1 && c.value == 100.into()));
	}

	#[test]
	pub(crate) fn should_update_structure() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);
		object.set_field(1, BinaryValue::from([1, 2, 3].as_ref())).unwrap();
		object.set_field(1, BinaryValue::from([4, 5, 6, 7].as_ref())).unwrap();

		let s: &BinaryValue = object.get_field(1).unwrap();
		assert_eq!(s.as_slice(), [4, 5, 6, 7]);
	}
}
