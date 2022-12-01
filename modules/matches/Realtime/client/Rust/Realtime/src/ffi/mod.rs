use std::sync::mpsc::SendError;
use std::sync::Mutex;

use lazy_static::lazy_static;
use thiserror::Error;

use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
use cheetah_matches_realtime_common::commands::field::FieldId;
use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
use cheetah_matches_realtime_common::commands::{CommandTypeId, FieldType, FieldValue};
use cheetah_matches_realtime_common::constants::GameObjectTemplateId;
use cheetah_matches_realtime_common::room::object::GameObjectId;
use cheetah_matches_realtime_common::room::owner::GameObjectOwner;
use cheetah_matches_realtime_common::room::RoomMemberId;

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
	#[error("Create client error {}",.0)]
	CreateClientError(String),
	#[error("Registry mutex error {}",.0)]
	RegistryMutex(String),
	#[error("Client not found {}",.0)]
	ClientNotFound(ClientId),
	#[error("Connection status mutex error {}",.0)]
	ConnectionStatusMutexError(String),
	#[error("Send task error {}",.source)]
	SendTaskError {
		#[from]
		source: SendError<ClientRequest>,
	},
}

impl ClientError {
	pub(crate) fn store_error_and_get_code(&self) -> u8 {
		let mut last_error = LAST_ERROR.lock().unwrap();
		let msg = format!("{:?}", self);
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
			let error = ClientError::RegistryMutex(format!("{:?}", e));
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
	object_id: GameObjectIdFFI,
	game_object_template_id: GameObjectTemplateId,
	field_id: FieldId,
	field_type: FieldTypeFFI,
	long_value_old: i64,
	long_value_new: i64,
	long_value_reset: i64,
	float_value_new: f64,
	binary_value_old: BufferFFI,
	binary_value_new: BufferFFI,
	binary_value_reset: BufferFFI,
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
			field_type: FieldTypeFFI::Long,
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
				ffi_command.long_value_reset = c.reset.unwrap_or_default();
			}
			C2SCommand::SetField(c) => match c.value {
				FieldValue::Long(v) => {
					ffi_command.long_value_new = v;
				}
				FieldValue::Double(v) => {
					ffi_command.float_value_new = v;
				}
				FieldValue::Structure(v) => {
					ffi_command.binary_value_new = v.into();
				}
			},
			C2SCommand::IncrementDouble(c) => {
				ffi_command.float_value_new = c.increment;
			}
			C2SCommand::CompareAndSetStructure(c) => {
				ffi_command.binary_value_old = From::from(&c.current);
				ffi_command.binary_value_new = From::from(&c.new);
				ffi_command.binary_value_reset = From::from(&c.reset.unwrap_or_default());
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

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObjectIdFFI {
	id: u32,
	pub room_owner: bool,
	member_id: RoomMemberId,
}

impl Default for GameObjectIdFFI {
	fn default() -> Self {
		GameObjectId::default().into()
	}
}

impl From<GameObjectId> for GameObjectIdFFI {
	fn from(source: GameObjectId) -> Self {
		(&source).into()
	}
}

impl From<&GameObjectId> for GameObjectIdFFI {
	fn from(from: &GameObjectId) -> Self {
		match from.owner {
			GameObjectOwner::Room => GameObjectIdFFI {
				id: from.id,
				room_owner: true,
				member_id: RoomMemberId::MAX,
			},
			GameObjectOwner::Member(member_id) => GameObjectIdFFI {
				id: from.id,
				room_owner: false,
				member_id,
			},
		}
	}
}

impl From<&GameObjectIdFFI> for GameObjectId {
	fn from(from: &GameObjectIdFFI) -> Self {
		if from.room_owner {
			Self {
				owner: GameObjectOwner::Room,
				id: from.id,
			}
		} else {
			Self {
				owner: GameObjectOwner::Member(from.member_id),
				id: from.id,
			}
		}
	}
}

const BUFFER_MAX_SIZE: usize = 255;

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BufferFFI {
	pub len: u8,
	pub pos: u8, // используется в C#
	pub buffer: [u8; BUFFER_MAX_SIZE],
}

impl Default for BufferFFI {
	fn default() -> Self {
		Self {
			len: 0,
			buffer: [0; BUFFER_MAX_SIZE],
			pos: 0,
		}
	}
}

impl From<Vec<u8>> for BufferFFI {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: Vec<u8>) -> Self {
		let mut buffer = BufferFFI {
			len: source.len() as u8,
			..Default::default()
		};
		buffer.buffer[0..source.len()].copy_from_slice(source.as_slice());
		buffer
	}
}

impl From<&BufferFFI> for BinaryValue {
	fn from(source: &BufferFFI) -> Self {
		BinaryValue::from(&source.buffer[0..source.len as usize])
	}
}

impl From<&BinaryValue> for BufferFFI {
	#[allow(clippy::cast_possible_truncation)]
	fn from(source: &BinaryValue) -> Self {
		let mut result = BufferFFI {
			len: source.len() as u8,
			pos: 0,
			buffer: [0; BUFFER_MAX_SIZE],
		};
		let buffer = &mut result.buffer[0..source.len()];
		buffer.copy_from_slice(source.as_slice());
		result
	}
}

impl From<BinaryValue> for BufferFFI {
	fn from(mut b: BinaryValue) -> Self {
		let len = b.len().try_into().expect("BinaryValue size exceeds u8");
		b.0.resize_default(b.0.capacity()).expect("unexpected buffer size");
		Self {
			len,
			pos: 0,
			buffer: b.0.into_array().expect("unexpected buffer size"),
		}
	}
}

#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldTypeFFI {
	Long,
	Double,
	Structure,
	Event,
}

impl From<FieldType> for FieldTypeFFI {
	fn from(source: FieldType) -> Self {
		(&source).into()
	}
}

impl From<&FieldType> for FieldTypeFFI {
	fn from(source: &FieldType) -> Self {
		match source {
			FieldType::Long => FieldTypeFFI::Long,
			FieldType::Double => FieldTypeFFI::Double,
			FieldType::Structure => FieldTypeFFI::Structure,
			FieldType::Event => FieldTypeFFI::Event,
		}
	}
}

impl From<FieldTypeFFI> for FieldType {
	fn from(source: FieldTypeFFI) -> Self {
		match source {
			FieldTypeFFI::Long => FieldType::Long,
			FieldTypeFFI::Double => FieldType::Double,
			FieldTypeFFI::Structure => FieldType::Structure,
			FieldTypeFFI::Event => FieldType::Event,
		}
	}
}

#[cfg(test)]
mod tests {
	use cheetah_matches_realtime_common::commands::binary_value::BinaryValue;
	use cheetah_matches_realtime_common::commands::c2s::C2SCommand;
	use cheetah_matches_realtime_common::commands::types::create::{C2SCreatedGameObjectCommand, CreateGameObjectCommand};
	use cheetah_matches_realtime_common::commands::types::delete::DeleteGameObjectCommand;
	use cheetah_matches_realtime_common::commands::types::event::{EventCommand, TargetEventCommand};
	use cheetah_matches_realtime_common::commands::types::field::{DeleteFieldCommand, SetFieldCommand};
	use cheetah_matches_realtime_common::commands::types::float::IncrementDoubleC2SCommand;
	use cheetah_matches_realtime_common::commands::types::forwarded::ForwardedCommand;
	use cheetah_matches_realtime_common::commands::types::long::{CompareAndSetLongCommand, IncrementLongC2SCommand};
	use cheetah_matches_realtime_common::commands::types::structure::CompareAndSetStructureCommand;
	use cheetah_matches_realtime_common::commands::{CommandTypeId, FieldType, FieldValue};
	use cheetah_matches_realtime_common::room::access::AccessGroups;
	use cheetah_matches_realtime_common::room::object::GameObjectId;
	use cheetah_matches_realtime_common::room::owner::GameObjectOwner;

	use crate::ffi::{BufferFFI, FieldTypeFFI, ForwardedCommandFFI, GameObjectIdFFI};

	#[test]
	fn should_convert_object_id() {
		let object_id = GameObjectId {
			owner: GameObjectOwner::Member(123),
			id: 100,
		};
		let object_id_fff = GameObjectIdFFI::from(&object_id);
		let converted_object_id = GameObjectId::from(&object_id_fff);
		assert_eq!(object_id, converted_object_id);
	}

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
					object_id: object_id.into(),
					game_object_template_id: 1,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CreatedGameObject(C2SCreatedGameObjectCommand {
						object_id,
						room_owner: false,
						singleton_key: None,
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CreatedGameObject,
					object_id: object_id.into(),
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
					object_id: object_id.into(),
					field_id,
					field_type: FieldTypeFFI::Long,
					long_value_new: 1,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CompareAndSetLong(CompareAndSetLongCommand {
						object_id,
						field_id,
						current: 1,
						new: 2,
						reset: Some(3),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CompareAndSetLong,
					object_id: object_id.into(),
					field_id,
					field_type: FieldTypeFFI::Long,
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
					field_type: FieldTypeFFI::Long,
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
					field_type: FieldTypeFFI::Double,
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
						value: FieldValue::Structure(b1.0.to_vec()),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::SetStructure,
					object_id: object_id.into(),
					field_id,
					field_type: FieldTypeFFI::Structure,
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
					field_type: FieldTypeFFI::Double,
					float_value_new: 1.2,
					..Default::default()
				},
			),
			(
				ForwardedCommand {
					creator,
					c2s: C2SCommand::CompareAndSetStructure(CompareAndSetStructureCommand {
						object_id,
						field_id,
						current: b1.clone(),
						new: b2.clone(),
						reset: Some(b3.clone()),
					}),
				},
				ForwardedCommandFFI {
					creator,
					command_type_id: CommandTypeId::CompareAndSetStructure,
					object_id: object_id.into(),
					field_id,
					field_type: FieldTypeFFI::Structure,
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
					field_type: FieldTypeFFI::Event,
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
					field_type: FieldTypeFFI::Event,
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
					field_type: FieldTypeFFI::Structure,
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

	#[test]
	fn test_buffer_ffi_from_binary_value() {
		let b = BinaryValue::from([1, 2, 3, 4].as_slice());
		let mut buffer: [u8; 255] = [0; 255];
		buffer[0] = 1;
		buffer[1] = 2;
		buffer[2] = 3;
		buffer[3] = 4;
		let ffi = BufferFFI { len: 4, pos: 0, buffer };
		assert_eq!(ffi, BufferFFI::from(b));
	}
}
