use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::commands::command::C2SCommandUnion;

impl Server2ClientFFIConverter for DeleteGameObjectCommand {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::Unload;
		ffi.object_id.set_from(&self.object_id);
	}
}


impl Client2ServerFFIConverter for DeleteGameObjectCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Unload);
		C2SCommandUnion::Delete(
			DeleteGameObjectCommand {
				object_id: ffi.object_id.to_common_game_object_id()
			})
	}
}

