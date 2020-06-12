use cheetah_relay_common::network::command::event::EventCommand;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for EventCommand {
	fn to_ffi(self, ffi: &mut CommandFFI) {
		ffi.command_type_s2c = S2CCommandFFIType::Event;
		ffi.object_id = self.global_object_id;
		ffi.field_id = self.field_id;
		ffi.event = From::from(self.event);
	}
}

impl Client2ServerFFIConverter for EventCommand {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Event);
		C2SCommandUnion::Event(
			EventCommand {
				global_object_id: ffi.object_id,
				field_id: ffi.field_id,
				event: From::from(ffi.event),
			})
	}
}