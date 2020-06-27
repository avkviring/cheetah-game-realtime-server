use core::fmt;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use cheetah_relay_common::constants::{MAX_FIELDS_IN_OBJECT, MAX_SIZE_STRUCT};
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::counters::Counters;
use crate::client::ffi::structures::Structures;
use crate::client::ffi::bytes::Bytes;

pub mod structures;
pub mod counters;
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
	pub structure: Bytes,
	pub event: Bytes,
	pub long_value: i64,
	pub float_value: f64,
	pub access_group: u64,
	pub long_counters: Counters<i64>,
	pub float_counters: Counters<f64>,
	pub structures: Structures,
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
	pub client: u16,
	pub id_type: ObjectIdType,
}

#[repr(C)]
#[derive(Debug)]
pub enum ObjectIdType {
	Root,
	Current,
	Client,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum S2CCommandFFIType {
	Upload,
	SetLongCounter,
	SetFloatCounter,
	Structure,
	Event,
	Unload,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum C2SCommandFFIType {
	Upload,
	IncrementLongCounter,
	SetLongCounter,
	IncrementFloatCounter,
	SetFloatCounter,
	Structure,
	Event,
	Unload,
}


impl Default for Command {
	fn default() -> Self {
		Command {
			command_type_s2c: S2CCommandFFIType::Unload,
			command_type_c2s: C2SCommandFFIType::Unload,
			object_id: Default::default(),
			field_id: Default::default(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			structure: Default::default(),
			event: Default::default(),
			long_value: Default::default(),
			float_value: Default::default(),
			access_group: Default::default(),
		}
	}
}

impl Default for ObjectId {
	fn default() -> Self {
		ObjectId {
			id: 0,
			client: 0,
			id_type: ObjectIdType::Root,
		}
	}
}

impl ObjectId {
	pub fn set_from(&mut self, id: &ClientGameObjectId) {
		self.id = id.id;
		match id.owner {
			ClientOwner::Root => { self.id_type = ObjectIdType::Root }
			ClientOwner::CurrentClient => {
				self.id_type = ObjectIdType::Current;
			}
			ClientOwner::Client(client) => {
				self.id_type = ObjectIdType::Client;
				self.client = client
			}
		}
	}
	
	pub fn to_common_game_object_id(&self) -> ClientGameObjectId {
		ClientGameObjectId {
			owner: match self.id_type {
				ObjectIdType::Root => { ClientOwner::Root }
				ObjectIdType::Current => { ClientOwner::CurrentClient }
				ObjectIdType::Client => { ClientOwner::Client(self.client) }
			},
			id: self.id,
		}
	}
}