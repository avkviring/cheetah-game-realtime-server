use core::fmt;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use fnv::FnvBuildHasher;

use cheetah_relay_common::constants::MAX_FIELDS_IN_OBJECT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Values<T> where T: Default {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub values: [T; MAX_FIELDS_IN_OBJECT],
}

impl<T> Debug for Values<T> where T: Default {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f
			.debug_struct("$name")
			.field("size", &self.count)
			.finish()
	}
}

impl<T> Default for Values<T> where T: Default + Copy {
	fn default() -> Self {
		Values {
			count: Default::default(),
			fields: [Default::default(); MAX_FIELDS_IN_OBJECT],
			values: [Default::default(); MAX_FIELDS_IN_OBJECT],
		}
	}
}

impl<IN: Clone, OUT: Default + From<IN> + Copy> From<&HashMap<u16, IN, FnvBuildHasher>> for Values<OUT> {
	fn from(value: &HashMap<u16, IN, FnvBuildHasher>) -> Self {
		let mut result: Values<OUT> = Default::default();
		result.count = value.len() as u8;
		for (i, (key, value)) in value.iter().enumerate() {
			result.fields[i] = *key;
			result.values[i] = From::<IN>::from(value.clone())
		};
		result
	}
}

impl<IN: Default + Clone, OUT: From<IN>> From<&Values<IN>> for HashMap<u16, OUT, FnvBuildHasher> {
	fn from(value: &Values<IN>) -> Self {
		let mut result = HashMap::<u16, OUT, FnvBuildHasher>::default();
		for i in 0..value.count as usize {
			let field = value.fields[i];
			let value = From::<IN>::from(value.values[i].clone());
			result.insert(field, value);
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	
	use fnv::{FnvBuildHasher, FnvHashMap};
	
	use crate::client::ffi::values::Values;
	
	#[test]
	fn should_convert_values() {
		let mut source = FnvHashMap::default();
		source.insert(10 as u16, 255 as u8);
		source.insert(20 as u16, 255 as u8);
		let fields = Values::<u8>::from(&source);
		let converted = HashMap::<u16, u8, FnvBuildHasher>::from(&fields);
		assert_eq!(source, converted);
	}
}