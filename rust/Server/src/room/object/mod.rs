use std::fmt::Debug;

use thiserror::Error;

use cheetah_common::commands::binary_value::Buffer;
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::types::create::{CreateGameObjectCommand, GameObjectCreatedS2CCommand};
use cheetah_common::commands::types::float::SetDoubleCommand;
use cheetah_common::commands::types::long::SetLongCommand;
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::constants::GameObjectTemplateId;
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;
use fields::Fields;

pub mod fields;

pub const MAX_FIELD_COUNT: usize = 64;

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
	pub doubles: Fields<f64>,
	pub longs: Fields<i64>,
	pub structures: Fields<Buffer>,
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
			doubles: Default::default(),
			longs: Default::default(),
			structures: Default::default(),
		}
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
		self.longs
			.collect_commands(commands, member_id, |field_id, value| S2CCommand::SetLong(SetLongCommand { object_id: self.id, field_id, value }))?;

		self.doubles
			.collect_commands(commands, member_id, |field_id, value| S2CCommand::SetDouble(SetDoubleCommand { object_id: self.id, field_id, value }))?;

		self.structures.collect_commands(commands, member_id, |field_id, value| {
			S2CCommand::SetStructure(SetStructureCommand { object_id: self.id, field_id, value })
		})?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::binary_value::Buffer;
	use cheetah_common::commands::field::Field;
	use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
	use cheetah_common::commands::types::create::GameObjectCreatedS2CCommand;
	use cheetah_common::commands::types::float::SetDoubleCommand;
	use cheetah_common::commands::types::long::SetLongCommand;
	use cheetah_common::commands::types::structure::SetStructureCommand;
	use cheetah_common::commands::FieldType;
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::room::object::{CreateCommandsCollector, GameObject};

	///
	/// Проверяем что все типы данных преобразованы в команды
	///
	#[test]
	pub(crate) fn should_collect_command() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 55, AccessGroups(63), true);
		object.longs.set(1, 100);
		object.doubles.set(2, 200.200);
		object.structures.set(1, [1, 2, 3].as_ref().into());

		let mut commands = CreateCommandsCollector::new();
		object.collect_create_commands(&mut commands, u16::MAX);

		assert!(matches!(
			&commands[0],
			S2CCommandWithMeta { field: None, creator: u16::MAX, command:S2CCommand::Create(c) }
			if c.object_id==id
			&& c.template == object.template_id
			&& c.access_groups == object.access_groups
		));

		assert_eq!(
			commands[1].command,
			S2CCommand::SetLong(SetLongCommand {
				object_id: id,
				field_id: 1,
				value: 100,
			})
		);

		assert_eq!(
			commands[2].command,
			S2CCommand::SetDouble(SetDoubleCommand {
				object_id: id,
				field_id: 2,
				value: 200.200,
			})
		);

		assert_eq!(
			commands[3].command,
			S2CCommand::SetStructure(SetStructureCommand {
				object_id: id,
				field_id: 1,
				value: [1, 2, 3].as_ref().into(),
			})
		);

		assert_eq!(commands[4].command, S2CCommand::Created(GameObjectCreatedS2CCommand { object_id: id }));
	}

	///
	/// Для несозданного объекта не должно быть команды Created
	///
	#[test]
	pub(crate) fn should_collect_command_for_not_created_object() {
		let id = GameObjectId::new(1, GameObjectOwner::Room);
		let mut object = GameObject::new(id, 0, Default::default(), false);
		object.longs.set(1, 100);

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
				command:S2CCommand::SetLong(c)
			}
			if c.object_id==id && c.field_id== 1 && c.value == 100));
	}

	#[test]
	pub(crate) fn should_update_structure() {
		let mut object = GameObject::new(GameObjectId::default(), 0, Default::default(), false);
		object.structures.set(1, Buffer::from([1, 2, 3].as_ref()));
		object.structures.set(1, Buffer::from([4, 5, 6, 7].as_ref()));

		let s: &Buffer = object.structures.get(1).unwrap();
		assert_eq!(s.as_slice(), [4, 5, 6, 7]);
	}
}
