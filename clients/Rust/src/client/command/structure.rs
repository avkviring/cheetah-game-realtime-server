use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};

#[derive(Debug)]
pub struct SetStructC2S {
	pub object_id: u64,
	pub field_id: u16,
	pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct SetStructS2C {
	pub object_id: u64,
	pub field_id: u16,
	pub data: Vec<u8>,
}

impl S2CCommandFFICollector for SetStructS2C {
	fn collect(self, command: &mut S2CCommandFFI) {
		command.s2c_command_type = S2CCommandFFIType::SetStruct;
		command.object_id = self.object_id;
		command.field_id = self.field_id;
		command.structure = From::from(self.data);
	}
}

impl SetStructC2S {
	pub fn from(command: S2CCommandFFI) -> C2SCommandUnion {
		debug_assert!(command.c2s_command_type == C2SCommandFFIType::SetStruct);
		C2SCommandUnion::SetStruct(
			SetStructC2S {
				object_id: command.object_id,
				field_id: command.field_id,
				data: From::from(command.structure),
			})
	}
}