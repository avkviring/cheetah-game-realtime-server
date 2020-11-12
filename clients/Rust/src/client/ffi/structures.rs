use core::fmt;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use fnv::FnvBuildHasher;

use cheetah_relay_common::constants::{ALL_STRUCTURES_SIZE, FieldID, MAX_FIELDS_IN_OBJECT, MAX_SIZE_STRUCT};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Structures {
	pub count: u8,
	pub fields: [u16; MAX_FIELDS_IN_OBJECT],
	pub sizes: [u8; MAX_FIELDS_IN_OBJECT],
	pub values: [u8; ALL_STRUCTURES_SIZE],
}


impl Default for Structures {
	fn default() -> Self {
		Structures {
			count: Default::default(),
			fields: [0; MAX_FIELDS_IN_OBJECT],
			sizes: [0; MAX_FIELDS_IN_OBJECT],
			values: [0; ALL_STRUCTURES_SIZE],
		}
	}
}

impl Debug for Structures {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		let mut debug_struct = formatter.debug_struct("Structures");
		debug_struct.field("count", &self.count);
		
		for i in 0..self.count {
			debug_struct.field(format!("f[{}]", i).as_str(), &self.fields[i as usize]);
		}
		for i in 0..self.count as usize {
			let start_offset = i * MAX_SIZE_STRUCT;
			let end_offset = start_offset + self.sizes[i] as usize;
			debug_struct.field(format!("data[{}]", i).as_str(), &hex::encode_upper(&self.values[start_offset..end_offset]));
		}
		
		
		debug_struct.finish()
	}
}


impl From<&HashMap<FieldID, Vec<u8>, FnvBuildHasher>> for Structures {
	fn from(from: &HashMap<u16, Vec<u8>, FnvBuildHasher>) -> Self {
		let mut structures: Structures = Default::default();
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

impl From<&Structures> for HashMap<FieldID, Vec<u8>, FnvBuildHasher> {
	fn from(from: &Structures) -> Self {
		let mut result = HashMap::default();
		for index in 0..from.count as usize {
			let start_offset = index * MAX_SIZE_STRUCT;
			let end_offset = start_offset + from.sizes[index] as usize;
			let value = from.values[start_offset..end_offset].to_vec();
			result.insert(from.fields[index], value);
		}
		result
	}
}


