use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, S2CCommandFFI, S2CCommandFFICollector, S2CCommandFFIType};

#[derive(Debug)]
pub struct IncrementFloatCounterC2S {
	pub object_id: u64,
	pub field_id: u16,
	pub increment: f64,
}

#[derive(Debug)]
pub struct SetFloatCounterS2C {
	pub object_id: u64,
	pub field_id: u16,
	pub value: f64,
}

impl S2CCommandFFICollector for SetFloatCounterS2C {
	fn collect(self, command: &mut S2CCommandFFI) {
		command.s2c_command_type = S2CCommandFFIType::SetFloatCounter;
		command.object_id = self.object_id;
		command.field_id = self.field_id;
		command.float_value = self.value;
	}
}

impl IncrementFloatCounterC2S {
	pub fn from(command: S2CCommandFFI) -> C2SCommandUnion {
		debug_assert!(command.c2s_command_type == C2SCommandFFIType::IncrementFloatCounter);
		C2SCommandUnion::IncrementFloatCounter(
			IncrementFloatCounterC2S {
				object_id: command.object_id,
				field_id: command.field_id,
				increment: command.float_value,
			})
	}
}