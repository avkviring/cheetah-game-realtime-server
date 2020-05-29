use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};

#[derive(Debug)]
pub struct IncrementLongCounterC2S {
	pub object_id: u64,
	pub field_id: u16,
	pub increment: i64,
}

#[derive(Debug)]
pub struct SetLongCounterS2C {
	pub object_id: u64,
	pub field_id: u16,
	pub value: i64,
}

impl S2CCommandFFICollector for SetLongCounterS2C {
	fn collect(self, command: &mut S2CCommandFFI) {
		command.s2c_command_type = S2CCommandFFIType::SetLongCounter;
		command.object_id = self.object_id;
		command.field_id = self.field_id;
		command.long_value = self.value;
	}
}

impl IncrementLongCounterC2S {
	pub fn from(command: S2CCommandFFI) -> C2SCommandUnion {
		debug_assert!(command.c2s_command_type == C2SCommandFFIType::IncrementLongCounter);
		C2SCommandUnion::IncrementLongCounter(
			IncrementLongCounterC2S {
				object_id: command.object_id,
				field_id: command.field_id,
				increment: command.long_value,
			})
	}
}