use cheetah_relay_common::commands::command::event::EventCommand;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for EventCommand {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::Event;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.event = From::from(self.event);
	}
}

impl Client2ServerFFIConverter for EventCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Event);
		C2SCommandUnion::Event(
			EventCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				event: From::from(ffi.event),
			})
	}
}