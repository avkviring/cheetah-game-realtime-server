use cheetah_relay_common::network::command::structure::SetStructCommand;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for SetStructCommand {
	fn to_ffi(self, command: &mut CommandFFI) {
		command.command_type_s2c = S2CCommandFFIType::SetStruct;
		command.object_id = self.global_object_id;
		command.field_id = self.field_id;
		command.structure = From::from(self.data);
	}
}

impl Client2ServerFFIConverter for SetStructCommand {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetStruct);
		C2SCommandUnion::SetStruct(
			SetStructCommand {
				global_object_id: ffi.object_id,
				field_id: ffi.field_id,
				data: From::from(ffi.structure),
			})
	}
}

