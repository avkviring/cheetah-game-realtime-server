use cheetah_relay_common::network::command::structure::StructureCommand;

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::network::command::C2SCommandUnion;

impl Server2ClientFFIConverter for StructureCommand {
	fn to_ffi(self, command: &mut Command) {
		command.command_type_s2c = S2CCommandFFIType::Structure;
		command.object_id.set_from(&self.object_id);
		command.field_id = self.field_id;
		command.structure = From::from(self.structure);
	}
}

impl Client2ServerFFIConverter for StructureCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Structure);
		C2SCommandUnion::Structure(
			StructureCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				structure: From::from(ffi.structure),
			})
	}
}

