use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for SetFloatCounterCommand {
	fn to_ffi(self, ffi: &mut CommandFFI) {
		ffi.command_type_s2c = S2CCommandFFIType::SetFloatCounter;
		ffi.object_id = self.global_object_id;
		ffi.field_id = self.field_id;
		ffi.float_value = self.value;
	}
}

impl Client2ServerFFIConverter for SetFloatCounterCommand {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetFloatCounter);
		C2SCommandUnion::SetFloatCounter(
			SetFloatCounterCommand {
				global_object_id: ffi.object_id,
				field_id: ffi.field_id,
				value: ffi.float_value,
			})
	}
}

impl Client2ServerFFIConverter for IncrementFloatCounterC2SCommand {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::IncrementFloatCounter);
		C2SCommandUnion::IncrementFloatCounter(
			IncrementFloatCounterC2SCommand {
				global_object_id: ffi.object_id,
				field_id: ffi.field_id,
				increment: ffi.float_value,
			})
	}
}