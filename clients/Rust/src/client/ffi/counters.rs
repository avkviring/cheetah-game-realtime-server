use core::fmt;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use cheetah_relay_common::constants::MAX_FIELDS_IN_OBJECT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Counters<T> where T: Default {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub values: [T; MAX_FIELDS_IN_OBJECT],
}

impl<T> Debug for Counters<T> where T: Default {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f
			.debug_struct("$name")
			.field("size", &self.count)
			.finish()
	}
}

impl<T> Default for Counters<T> where T: Default + Copy {
	fn default() -> Self {
		Counters {
			count: Default::default(),
			fields: [Default::default(); MAX_FIELDS_IN_OBJECT],
			values: [Default::default(); MAX_FIELDS_IN_OBJECT],
		}
	}
}

impl<IN: Clone, OUT: Default + From<IN> + Copy> From<&HashMap<u16, IN>> for Counters<OUT> {
	fn from(value: &HashMap<u16, IN>) -> Self {
		let mut result: Counters<OUT> = Default::default();
		result.count = value.len() as u8;
		for (i, (key, value)) in value.into_iter().enumerate() {
			result.fields[i] = key.clone();
			result.values[i] = From::<IN>::from(value.clone())
		};
		result
	}
}

impl<IN: Default + Clone, OUT: From<IN>> From<&Counters<IN>> for HashMap<u16, OUT> {
	fn from(value: &Counters<IN>) -> Self {
		let mut result = HashMap::<u16, OUT>::new();
		for i in 0..value.count as usize {
			let field = value.fields[i].clone();
			let value = From::<IN>::from(value.values[i].clone());
			result.insert(field, value);
		}
		result
	}
}
