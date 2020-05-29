use std::collections::HashMap;

use cheetah_relay_common::constants::{MAX_FIELDS_IN_OBJECT, MAX_SIZE_STRUCT};

///
/// Структура для обмена данными с C#
/// фактически - эмуляция union
/// используется в единственном экземпляре
///
#[repr(C)]
pub struct S2CCommandFFI {
	pub s2c_command_type: S2CCommandFFIType,
	pub c2s_command_type: C2SCommandFFIType,
	pub object_id: u64,
	pub field_id: u16,
	pub long_counters: FieldsFFI<i64>,
	pub float_counters: FieldsFFI<f64>,
	pub structures: FieldsFFI<FieldFFIBinary>,
	pub structure: FieldFFIBinary,
	pub event: FieldFFIBinary,
	pub long_value: i64,
	pub float_value: f64,
}

///
/// Заполнение FFI структуры данными для произвольной команды
///
pub trait S2CCommandFFICollector {
	fn collect(self, command: &mut S2CCommandFFI);
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum S2CCommandFFIType {
	Upload,
	SetLongCounter,
	SetFloatCounter,
	SetStruct,
	ReceiveEvent,
	Unload,
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum C2SCommandFFIType {
	Upload,
	IncrementLongCounter,
	IncrementFloatCounter,
	SetStruct,
	SendEvent,
	Unload,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldFFIBinary {
	pub binary_size: usize,
	pub value: [u8; MAX_SIZE_STRUCT],
}

impl Default for FieldFFIBinary {
	fn default() -> Self {
		FieldFFIBinary {
			binary_size: 0,
			value: [0; MAX_SIZE_STRUCT],
		}
	}
}


#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldsFFI<T> where T: Default {
	pub size: usize,
	pub values: [FieldFFI<T>; MAX_FIELDS_IN_OBJECT],
}


impl<T> Default for FieldsFFI<T> where T: Default + Copy {
	fn default() -> Self {
		FieldsFFI {
			size: Default::default(),
			values: [Default::default(); MAX_FIELDS_IN_OBJECT],
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FieldFFI<T> where T: Default {
	pub field_id: u16,
	pub values: T,
}

impl<T> Default for FieldFFI<T> where T: Default {
	fn default() -> FieldFFI<T> {
		FieldFFI {
			field_id: Default::default(),
			values: Default::default(),
		}
	}
}


impl Default for S2CCommandFFI {
	fn default() -> Self {
		S2CCommandFFI {
			s2c_command_type: S2CCommandFFIType::None,
			c2s_command_type: C2SCommandFFIType::None,
			object_id: Default::default(),
			field_id: Default::default(),
			long_counters: Default::default(),
			float_counters: Default::default(),
			structures: Default::default(),
			structure: Default::default(),
			event: Default::default(),
			long_value: Default::default(),
			float_value: Default::default(),
		}
	}
}

///
/// Конвертируем HashMap в FieldsFFI - так как HashMap не передать в C#
///
impl<IN: Clone, OUT: Default + From<IN> + Copy> From<&HashMap<u16, IN>> for FieldsFFI<OUT> {
	fn from(value: &HashMap<u16, IN>) -> Self {
		let mut result: FieldsFFI<OUT> = Default::default();
		result.size = value.len();
		for (i, (key, value)) in value.into_iter().enumerate() {
			let mut field = &mut result.values[i];
			field.field_id = key.clone();
			field.values = From::<IN>::from(value.clone());
		};
		result
	}
}

impl From<Vec<u8>> for FieldFFIBinary {
	fn from(value: Vec<u8>) -> Self {
		let mut result: FieldFFIBinary = Default::default();
		result.binary_size = value.len();
		result.value[0..value.len()].copy_from_slice(&value);
		result
	}
}

impl From<FieldFFIBinary> for Vec<u8> {
	fn from(value: FieldFFIBinary) -> Self {
		Vec::from(&value.value[0..value.binary_size])
	}
}

impl<IN: Default + Clone, OUT: From<IN>> From<FieldsFFI<IN>> for HashMap<u16, OUT> {
	fn from(value: FieldsFFI<IN>) -> Self {
		let mut result = HashMap::<u16, OUT>::new();
		value.values[0..value.size].iter().for_each(|v| {
			let key = v.field_id;
			let value = From::<IN>::from(v.values.clone());
			result.insert(key, value);
		});
		result
	}
}