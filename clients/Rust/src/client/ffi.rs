use core::fmt;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use cheetah_relay_common::constants::{MAX_FIELDS_IN_OBJECT, MAX_SIZE_STRUCT};
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use crate::client::command::C2SCommandUnion;

///
/// Структура для обмена данными с C#
/// фактически - эмуляция union
/// используется в единственном экземпляре
///
#[repr(C)]
#[derive(Debug)]
pub struct CommandFFI {
	pub command_type_s2c: S2CCommandFFIType,
	pub command_type_c2s: C2SCommandFFIType,
	pub field_id: u16,
	pub object_id: ObjectId,
	pub structure: FieldFFIBinary,
	pub event: FieldFFIBinary,
	pub long_value: i64,
	pub float_value: f64,
	pub access_group: u64,
	pub long_counters: FieldsFFI<i64>,
	pub float_counters: FieldsFFI<f64>,
	pub structures: FieldsFFI<FieldFFIBinary>,
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


///
/// Конвертер команды в FFI структуру
///
pub trait Server2ClientFFIConverter {
	fn to_ffi(self, ffi: &mut CommandFFI);
}


///
/// Конвертер FFI структуры в команду
pub trait Client2ServerFFIConverter {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion;
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldFFIBinary {
	pub count: u8,
	pub value: [u8; MAX_SIZE_STRUCT],
}


impl FieldFFIBinary {
	pub fn as_slice(&self) -> &[u8] {
		&self.value[0..self.count as usize]
	}
}

impl Debug for FieldFFIBinary {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f
			.debug_struct("$name")
			.field("size", &self.count)
			.finish()
	}
}

impl Default for FieldFFIBinary {
	fn default() -> Self {
		FieldFFIBinary {
			count: 0,
			value: [0; MAX_SIZE_STRUCT],
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldsFFI<T> where T: Default {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub values: [T; MAX_FIELDS_IN_OBJECT],
}

impl<T> Debug for FieldsFFI<T> where T: Default {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f
			.debug_struct("$name")
			.field("size", &self.count)
			.finish()
	}
}

impl<T> Default for FieldsFFI<T> where T: Default + Copy {
	fn default() -> Self {
		FieldsFFI {
			count: Default::default(),
			fields: [Default::default(); MAX_FIELDS_IN_OBJECT],
			values: [Default::default(); MAX_FIELDS_IN_OBJECT],
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldFFI<T> where T: Default {
	pub field_id: u16,
	pub value: T,
}

impl<T> Default for FieldFFI<T> where T: Default {
	fn default() -> FieldFFI<T> {
		FieldFFI {
			field_id: Default::default(),
			value: Default::default(),
		}
	}
}

impl Default for CommandFFI {
	fn default() -> Self {
		CommandFFI {
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

///
/// Конвертируем HashMap в FieldsFFI - так как HashMap не передать в C#
///
impl<IN: Clone, OUT: Default + From<IN> + Copy> From<&HashMap<u16, IN>> for FieldsFFI<OUT> {
	fn from(value: &HashMap<u16, IN>) -> Self {
		let mut result: FieldsFFI<OUT> = Default::default();
		result.count = value.len() as u8;
		for (i, (key, value)) in value.into_iter().enumerate() {
			result.fields[i] = key.clone();
			result.values[i] = From::<IN>::from(value.clone())
		};
		result
	}
}

impl From<Vec<u8>> for FieldFFIBinary {
	fn from(value: Vec<u8>) -> Self {
		let mut result: FieldFFIBinary = Default::default();
		result.count = value.len() as u8;
		result.value[0..value.len()].copy_from_slice(&value);
		result
	}
}

impl From<FieldFFIBinary> for Vec<u8> {
	fn from(value: FieldFFIBinary) -> Self {
		Vec::from(&value.value[0..value.count as usize])
	}
}

impl<IN: Default + Clone, OUT: From<IN>> From<FieldsFFI<IN>> for HashMap<u16, OUT> {
	fn from(value: FieldsFFI<IN>) -> Self {
		let mut result = HashMap::<u16, OUT>::new();
		for i in 0..value.count as usize {
			let field = value.fields[i].clone();
			let value = From::<IN>::from(value.values[i].clone());
			result.insert(field, value);
		}
		result
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