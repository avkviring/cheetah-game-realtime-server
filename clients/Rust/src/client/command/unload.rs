use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};

#[derive(Debug)]
pub struct UnloadObjectC2S {
	pub object_id: u64
}

#[derive(Debug)]
pub struct UnloadObjectS2C {
	pub object_id: u64
}

impl S2CCommandFFICollector for UnloadObjectS2C {
	fn collect(self, command: &mut S2CCommandFFI) {
		command.s2c_command_type = S2CCommandFFIType::Unload;
		command.object_id = self.object_id;
	}
}


impl UnloadObjectC2S {
	pub fn from(command: S2CCommandFFI) -> C2SCommandUnion {
		debug_assert!(command.c2s_command_type == C2SCommandFFIType::Unload);
		C2SCommandUnion::Unload(
			UnloadObjectC2S {
				object_id: command.object_id
			})
	}
}
