use std::io;
use std::sync::mpsc::SendError;
use std::sync::Mutex;

use lazy_static::lazy_static;
use thiserror::Error;

use cheetah_common::commands::binary_value::BinaryValue;
use cheetah_common::commands::c2s::C2SCommand;
use cheetah_common::commands::field::FieldId;
use cheetah_common::commands::types::forwarded::ForwardedCommand;
use cheetah_common::commands::{CommandTypeId, FieldType, FieldValue};
use cheetah_common::constants::GameObjectTemplateId;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::RoomMemberId;

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::registry::{ClientId, Registry};
use crate::clients::ClientRequest;

pub mod channel;
pub mod client;
pub mod command;
pub mod logs;

#[derive(Error, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ClientError {
	#[error("Create client error {0}")]
	CreateClientError(#[from] io::Error),
	#[error("Registry mutex error {0}")]
	RegistryMutex(String),
	#[error("Client not found {0}")]
	ClientNotFound(ClientId),
	#[error("Connection status mutex error {0}")]
	ConnectionStatusMutexError(String),
	#[error("Send task error {0}")]
	SendTaskError(#[from] SendError<ClientRequest>),
}

impl ClientError {
	pub(crate) fn store_error_and_get_code(&self) -> u8 {
		let mut last_error = LAST_ERROR.lock().unwrap();
		let msg = format!("{self:?}");
		*last_error = msg;

		match self {
			ClientError::RegistryMutex(_) => 1,
			ClientError::ClientNotFound(_) => 2,
			ClientError::ConnectionStatusMutexError { .. } => 3,
			ClientError::SendTaskError { .. } => 4,
			ClientError::CreateClientError(_) => 5,
		}
	}
}

lazy_static! {
	static ref REGISTRY: Mutex<Registry> = Mutex::new(Default::default());
	static ref LAST_ERROR: Mutex<String> = Mutex::new(String::new());
}

pub fn execute<F, R>(body: F) -> u8
where
	F: FnOnce(&mut Registry) -> Result<R, ClientError>,
{
	let mut lock = REGISTRY.lock();
	match lock.as_mut() {
		Ok(registry) => match body(registry) {
			Ok(_) => 0,
			Err(e) => e.store_error_and_get_code(),
		},
		Err(e) => {
			let error = ClientError::RegistryMutex(format!("{e:?}"));
			error.store_error_and_get_code()
		}
	}
}

pub fn execute_with_client<F, R>(client_id: ClientId, action: F) -> u8
where
	F: FnOnce(&mut ApplicationThreadClient) -> Result<R, ClientError>,
{
	execute(|registry| match registry.clients.get_mut(&client_id) {
		None => Err(ClientError::ClientNotFound(client_id)),
		Some(client_api) => action(client_api),
	})
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct ForwardedCommandFFI {
	command_type_id: CommandTypeId,
	creator: RoomMemberId,
	target: RoomMemberId,
	object_id: GameObjectId,
	game_object_template_id: GameObjectTemplateId,
	field_id: FieldId,
	field_type: FieldType,
	long_value_old: i64,
	long_value_new: i64,
	long_value_reset: i64,
	float_value_new: f64,
	binary_value_old: BinaryValue,
	binary_value_new: BinaryValue,
	binary_value_reset: BinaryValue,
}

impl Default for ForwardedCommandFFI {
	fn default() -> Self {
		Self {
			command_type_id: CommandTypeId::DetachFromRoom,
			creator: 0,
			target: 0,
			object_id: Default::default(),
			game_object_template_id: 0,
			field_id: 0,
			field_type: FieldType::Long,
			long_value_old: 0,
			long_value_new: 0,
			long_value_reset: 0,
			float_value_new: 0.0,
			binary_value_old: Default::default(),
			binary_value_new: Default::default(),
			binary_value_reset: Default::default(),
		}
	}
}

impl From<ForwardedCommand> for ForwardedCommandFFI {
	fn from(c: ForwardedCommand) -> Self {
		let mut ffi_command = ForwardedCommandFFI {
			command_type_id: c.c2s.get_type_id(),
			creator: c.creator,
			object_id: c.c2s.get_object_id().unwrap_or_default().into(),
			field_id: c.c2s.get_field_id().unwrap_or_default(),
			field_type: c.c2s.get_field_type().unwrap_or(FieldType::Long).into(),
			..Default::default()
		};

		match c.c2s {
			C2SCommand::CreateGameObject(c) => {
				ffi_command.game_object_template_id = c.template;
			}
			C2SCommand::CreatedGameObject(_) => {}
			C2SCommand::IncrementLongValue(c) => {
				ffi_command.long_value_new = c.increment;
			}
			C2SCommand::CompareAndSetLong(c) => {
				ffi_command.long_value_old = c.current;
				ffi_command.long_value_new = c.new;
				ffi_command.long_value_reset = c.get_reset().unwrap_or_default();
			}
			C2SCommand::SetField(c) => match c.value {
				FieldValue::Long(v) => {
					ffi_command.long_value_new = v;
				}
				FieldValue::Double(v) => {
					ffi_command.float_value_new = v;
				}
				FieldValue::Structure(v) => {
					ffi_command.binary_value_new = v;
				}
			},
			C2SCommand::IncrementDouble(c) => {
				ffi_command.float_value_new = c.increment;
			}
			C2SCommand::CompareAndSetStructure(c) => {
				ffi_command.binary_value_old = c.current;
				ffi_command.binary_value_new = c.new;
				ffi_command.binary_value_reset = c.get_reset().cloned().unwrap_or_default();
			}
			C2SCommand::Event(c) => {
				ffi_command.binary_value_new = c.event.into();
			}
			C2SCommand::TargetEvent(c) => {
				ffi_command.target = c.target;
				ffi_command.binary_value_new = c.event.event.into();
			}
			C2SCommand::Delete(_) => {}
			C2SCommand::DeleteField(_) => {}
			C2SCommand::AttachToRoom => {}
			C2SCommand::DetachFromRoom => {}
			C2SCommand::Forwarded(_) => panic!("received invalid nested ForwardedCommand"),
		};

		ffi_command
	}
}

#[cfg(test)]
mod tests {
	use cheetah_common::commands::binary_value::BinaryValue;
	use cheetah_common::commands::c2s::C2SCommand;
	use cheetah_common::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
	use cheetah_common::commands::types::delete::DeleteGameObjectCommand;
	use cheetah_common::commands::types::event::{EventCommand, TargetEventCommand};
	use cheetah_common::commands::types::field::{DeleteFieldCommand, SetFieldCommand};
	use cheetah_common::commands::types::float::IncrementDoubleC2SCommand;
	use cheetah_common::commands::types::forwarded::ForwardedCommand;
	use cheetah_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
	use cheetah_common::commands::types::structure::CompareAndSetStructureCommand;
	use cheetah_common::commands::{CommandTypeId, FieldType, FieldValue};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::ffi::ForwardedCommandFFI;

	#[test]
	fn should_convert_forwarded_to_ffi() {
		let creator = 123;
		let object_id = GameObjectId::new(234, GameObjectOwner::Room);
		let field_id = 345;
		let target = 456;
		let b1 = BinaryValue::from([1, 2, 3, 4].as_slice());
		let b2 = BinaryValue::from([2, 3, 4].as_slice());
		let b3 = BinaryValue::from([3, 4].as_slice());

		let tests = [
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::AttachToRoom,
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::AttachToRoom,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::DetachFromRoom,
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::DetachFromRoom,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CreateGameObject(CreateGameObjectCommand {
						object_id,
						template: 1,
						access_groups: AccessGroups::super_group(),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CreateGameObject,
					object_id,
					game_object_template_id: 1,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand::new(object_id, false, None)),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CreatedGameObject,
					object_id,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::IncrementLongValue(IncrementLongC2SCommand {
						object_id,
						field_id,
						increment: 1,
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::IncrementLong,
					object_id,
					field_id,
					field_type: FieldType::Long,
					long_value_new: 1,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CompareAndSetLong(CompareAndSetLongCommand::new(object_id, field_id, 1, 2, Some(3))),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CompareAndSetLong,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Long,
					long_value_old: 1,
					long_value_new: 2,
					long_value_reset: 3,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::SetField(SetFieldCommand {
						object_id,
						field_id,
						value: FieldValue::Long(1),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::SetLong,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Long,
					long_value_new: 1,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::SetField(SetFieldCommand {
						object_id,
						field_id,
						value: FieldValue::Double(1.2),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::SetDouble,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Double,
					float_value_new: 1.2,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::SetField(SetFieldCommand {
						object_id,
						field_id,
						value: FieldValue::Structure(b1.as_slice().into()),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::SetStructure,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Structure,
					binary_value_new: b1.clone().into(),
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::IncrementDouble(IncrementDoubleC2SCommand {
						object_id,
						field_id,
						increment: 1.2,
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::IncrementDouble,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Double,
					float_value_new: 1.2,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand::new(
						object_id,
						field_id,
						b1.clone(),
						b2.clone(),
						Some(b3.clone()),
					)),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CompareAndSetStructure,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Structure,
					binary_value_old: b1.clone().into(),
					binary_value_new: b2.into(),
					binary_value_reset: b3.into(),
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::Event(EventCommand {
						object_id,
						field_id,
						event: b1.clone(),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::Event,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Event,
					binary_value_new: b1.clone().into(),
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::TargetEvent(TargetEventCommand {
						target,
						event: EventCommand {
							object_id,
							field_id,
							event: b1.clone(),
						},
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::TargetEvent,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Event,
					target,
					binary_value_new: b1.into(),
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::Delete(DeleteGameObjectCommand { object_id }),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::Delete,
					object_id: object_id.into(),
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::DeleteField(DeleteFieldCommand {
						field_id,
						object_id,
						field_type: FieldType::Structure,
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::DeleteField,
					object_id: object_id.into(),
					field_id,
					field_type: FieldType::Structure,
					..Default::default()
				},
			),
		];

		for (from, want) in tests {
			assert_eq!(want, from.into());
		}
	}

	#[test]
	#[should_panic(expected = "received invalid nested ForwardedCommand")]
	fn should_panic_on_nested_forwarded_command() {
		let _ = ForwardedCommandFFI::from(ForwardedCommand {
			creator: 0,
			c2s: C2SCommand::Forwarded(Box::new(ForwardedCommand {
				creator: 0,
				c2s: C2SCommand::AttachToRoom,
			})),
		});
	}
}
