use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::commands::command::C2SCommandUnion;

impl Server2ClientFFIConverter for SetLongCommand {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::SetLongCounter;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.long_value = self.value;
	}
}

impl Client2ServerFFIConverter for SetLongCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetLongCounter);
		C2SCommandUnion::SetLongCounter(
			SetLongCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				value: ffi.long_value,
			})
	}
}

impl Client2ServerFFIConverter for IncrementLongC2SCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::IncrementLongCounter);
		C2SCommandUnion::IncrementLongCounter(
			IncrementLongC2SCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				increment: ffi.long_value,
			})
	}
}