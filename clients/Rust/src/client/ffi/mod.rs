use std::fmt::Debug;

use cheetah_relay_common::commands::command::C2SCommandUnion;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::client::ffi::bytes::Bytes;
use crate::client::ffi::values::Values;
use crate::client::ffi::structures::Structures;

pub mod structures;
pub mod values;
pub mod bytes;


///
/// Структура для обмена данными с C#
/// фактически - эмуляция union
/// используется в единственном экземпляре
///
#[repr(C)]
#[derive(Debug)]
pub struct Command {
	pub command_type_s2c: S2CCommandFFIType,
	pub command_type_c2s: C2SCommandFFIType,
	pub field_id: u16,
	pub object_id: ObjectId,
	pub object_template: u16,
	pub structure: Bytes,
	pub event: Bytes,
	pub long_value: i64,
	pub float_value: f64,
	pub access_group: u64,
	pub longs: Values<i64>,
	pub floats: Values<f64>,
	pub structures: Structures,
	pub meta_timestamp: u64,
	pub meta_source_client: UserPublicKey,
}

///
/// Конвертер команды в FFI структуру
///
pub trait Server2ClientFFIConverter {
	fn to_ffi(self, ffi: &mut Command);
}


///
/// Конвертер FFI структуры в команду
pub trait Client2ServerFFIConverter {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion;
}


#[repr(C)]
#[derive(Debug)]
pub struct ObjectId {
	pub id: u32,
	pub user_public_key: UserPublicKey,
	pub id_type: ObjectIdType,
}

#[repr(C)]
#[derive(Debug)]
pub enum ObjectIdType {
	Root,
	User,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum S2CCommandFFIType {
	Create,
	SetLongCounter,
	SetFloatCounter,
	Structure,
	Event,
	Unload,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum C2SCommandFFIType {
	Create,
	IncrementLongCounter,
	SetLongCounter,
	IncrementFloatCounter,
	SetFloatCounter,
	Structure,
	Event,
	Unload,
	LoadRoom,
}


impl Default for Command {
	fn default() -> Self {
		Command {
			command_type_s2c: S2CCommandFFIType::Unload,
			command_type_c2s: C2SCommandFFIType::Unload,
			object_id: Default::default(),
			object_template: Default::default(),
			field_id: Default::default(),
			longs: Default::default(),
			floats: Default::default(),
			structures: Default::default(),
			structure: Default::default(),
			event: Default::default(),
			long_value: Default::default(),
			float_value: Default::default(),
			access_group: Default::default(),
			meta_timestamp: Default::default(),
			meta_source_client: Default::default(),
		}
	}
}

impl Default for ObjectId {
	fn default() -> Self {
		ObjectId {
			id: 0,
			user_public_key: 0,
			id_type: ObjectIdType::Root,
		}
	}
}

impl ObjectId {
	pub fn set_from(&mut self, id: &GameObjectId) {
		self.id = id.id;
		match id.owner {
			ClientOwner::Root => { self.id_type = ObjectIdType::Root }
			ClientOwner::User(user_public_key) => {
				self.id_type = ObjectIdType::User;
				self.user_public_key = user_public_key
			}
		}
	}
	
	pub fn to_common_game_object_id(&self) -> GameObjectId {
		GameObjectId {
			owner: match self.id_type {
				ObjectIdType::Root => { ClientOwner::Root }
				ObjectIdType::User => { ClientOwner::User(self.user_public_key) }
			},
			id: self.id,
		}
	}
}