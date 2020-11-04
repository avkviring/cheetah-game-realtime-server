use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for SetFloat64Command {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::SetFloatCounter;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.float_value = self.value;
	}
}

impl Client2ServerFFIConverter for SetFloat64Command {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetFloatCounter);
		C2SCommandUnion::SetFloatCounter(
			SetFloat64Command {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				value: ffi.float_value,
			})
	}
}

impl Client2ServerFFIConverter for IncrementFloat64C2SCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::IncrementFloatCounter);
		C2SCommandUnion::IncrementFloatCounter(
			IncrementFloat64C2SCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				increment: ffi.float_value,
			})
	}
}