use std::collections::HashMap;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, FieldsFFI, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};

#[derive(Debug)]
pub struct UploadObjectC2S {
	pub object_id: u64,
	pub long_counters: HashMap<u16, i64>,
	pub float_counters: HashMap<u16, f64>,
	pub structures: HashMap<u16, Vec<u8>>,
}

#[derive(Debug)]
pub struct UploadObjectS2C {
	pub object_id: u64,
	pub long_counters: HashMap<u16, i64>,
	pub float_counters: HashMap<u16, f64>,
	pub structures: HashMap<u16, Vec<u8>>,
}

impl S2CCommandFFICollector for UploadObjectS2C {
	fn collect(self, command: &mut S2CCommandFFI) {
		command.s2c_command_type = S2CCommandFFIType::Upload;
		command.object_id = self.object_id;
		command.long_counters = FieldsFFI::from(&self.long_counters);
		command.float_counters = FieldsFFI::from(&self.float_counters);
		command.structures = FieldsFFI::from(&self.structures);
	}
}

impl UploadObjectC2S {
	pub fn from(command: S2CCommandFFI) -> C2SCommandUnion {
		debug_assert!(command.c2s_command_type == C2SCommandFFIType::Upload);
		C2SCommandUnion::Upload(
			UploadObjectC2S {
				object_id: command.object_id,
				long_counters: From::from(command.long_counters),
				float_counters: From::from(command.float_counters),
				structures: From::from(command.structures),
			})
	}
}
