use std::collections::HashMap;

use as_slice::AsSlice;
use fnv::FnvBuildHasher;

use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::constants::FieldID;
use cheetah_relay_common::room::fields::{GameObjectFields, HeaplessBuffer};

use crate::ffi::{execute_with_client, GameObjectIdFFI};

/// Зарегистрировать обработчик загрузки нового игрового объекта
#[no_mangle]
pub extern fn set_create_object_listener(
	listener: extern fn(&S2CMetaCommandInformation, &GameObjectIdFFI, template: u16, fields: &GameObjectFieldsFFI)) -> bool {
	execute_with_client(|client| {
		client.register_create_object_listener(listener);
	}).is_ok()
}

#[no_mangle]
pub extern "C" fn create_object(
	template: u16,
	access_group: u64,
	fields: &GameObjectFieldsFFI,
	result: &mut GameObjectIdFFI,
) -> bool {
	execute_with_client(|client| {
		let game_object_id = client.create_game_object(template, access_group, From::from(fields));
		*result = game_object_id;
	}).is_ok()
}


pub const MAX_FIELDS_IN_OBJECT: usize = 255;
pub const MAX_SIZE_STRUCT: usize = 255;
pub const ALL_STRUCTURES_SIZE: usize = MAX_FIELDS_IN_OBJECT * MAX_SIZE_STRUCT;

#[repr(C)]
#[derive(Default)]
pub struct GameObjectFieldsFFI {
	pub structures: ObjectStructuresFFI,
	pub floats: ObjectValuesFFI<f64>,
	pub longs: ObjectValuesFFI<i64>,
}


impl From<&GameObjectFieldsFFI> for GameObjectFields {
	fn from(source: &GameObjectFieldsFFI) -> Self {
		Self {
			longs: From::from(&source.longs),
			floats: From::from(&source.floats),
			structures: From::from(&source.structures),
		}
	}
}

impl From<GameObjectFields> for GameObjectFieldsFFI {
	fn from(source: GameObjectFields) -> Self {
		Self {
			structures: From::from(&source.structures),
			floats: From::from(&source.floats),
			longs: From::from(&source.longs),
		}
	}
}


#[repr(C)]
pub struct ObjectStructuresFFI {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub sizes: [u8; MAX_FIELDS_IN_OBJECT],
	pub values: [u8; ALL_STRUCTURES_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ObjectValuesFFI<T> where T: Default {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub values: [T; MAX_FIELDS_IN_OBJECT],
}


impl Default for ObjectStructuresFFI {
	fn default() -> Self {
		Self {
			count: 0,
			fields: [0; MAX_FIELDS_IN_OBJECT],
			sizes: [0; MAX_FIELDS_IN_OBJECT],
			values: [0; ALL_STRUCTURES_SIZE],
		}
	}
}

impl From<&HashMap<FieldID, HeaplessBuffer, FnvBuildHasher>> for ObjectStructuresFFI {
	fn from(from: &HashMap<u16, HeaplessBuffer, FnvBuildHasher>) -> Self {
		let mut structures: ObjectStructuresFFI = Default::default();
		let mut index = 0;
		for (field, value) in from {
			structures.fields[index] = *field;
			structures.sizes[index] = value.len() as u8;
			let offset = index * MAX_SIZE_STRUCT;
			structures.values[offset..offset + value.len()].copy_from_slice(value.as_slice());
			index += 1;
		}
		structures.count = index as u8;
		structures
	}
}

impl From<&ObjectStructuresFFI> for HashMap<FieldID, HeaplessBuffer, FnvBuildHasher> {
	fn from(from: &ObjectStructuresFFI) -> Self {
		let mut result = HashMap::default();
		for index in 0..from.count as usize {
			let start_offset = index * MAX_SIZE_STRUCT;
			let end_offset = start_offset + from.sizes[index] as usize;
			let value = HeaplessBuffer::from_slice(&from.values[start_offset..end_offset]).unwrap();
			result.insert(from.fields[index], value);
		}
		result
	}
}


impl<T> Default for ObjectValuesFFI<T> where T: Default + Copy {
	fn default() -> Self {
		ObjectValuesFFI {
			count: Default::default(),
			fields: [Default::default(); MAX_FIELDS_IN_OBJECT],
			values: [Default::default(); MAX_FIELDS_IN_OBJECT],
		}
	}
}

impl<IN: Clone, OUT: Default + From<IN> + Copy> From<&heapless::FnvIndexMap<FieldID, IN, heapless::consts::U256>> for ObjectValuesFFI<OUT> {
	fn from(value: &heapless::FnvIndexMap<FieldID, IN, heapless::consts::U256>) -> Self {
		let mut result: ObjectValuesFFI<OUT> = Default::default();
		result.count = value.len() as u8;
		for (i, (key, value)) in value.iter().enumerate() {
			result.fields[i] = *key;
			result.values[i] = From::<IN>::from(value.clone())
		};
		result
	}
}

impl<IN: Default + Clone, OUT: From<IN>> From<&ObjectValuesFFI<IN>> for heapless::FnvIndexMap<FieldID, OUT, heapless::consts::U256> {
	fn from(value: &ObjectValuesFFI<IN>) -> Self {
		let mut result: heapless::FnvIndexMap<FieldID, OUT, heapless::consts::U256> = Default::default();
		for i in 0..value.count as usize {
			let field = value.fields[i];
			let value = From::<IN>::from(value.values[i].clone());
			result.insert(field, value).ok().unwrap();
		}
		result
	}
}


#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::fields::{GameObjectFields, HeaplessBuffer};
	
	use crate::ffi::command::create::GameObjectFieldsFFI;
	
	#[test]
	fn test_convert() {
		let mut source = GameObjectFields::default();
		source.floats.insert(5, 500.5).unwrap();
		source.longs.insert(1, 100).unwrap();
		let mut buffer = HeaplessBuffer::new();
		buffer.push(1).unwrap();
		source.structures.insert(3, buffer);
		
		let converted = GameObjectFieldsFFI::from(source.clone());
		let dest = GameObjectFields::from(&converted);
		assert_eq!(source, dest);
	}
}