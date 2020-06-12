use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for UnloadGameObjectCommand {
	fn to_ffi(self, ffi: &mut CommandFFI) {
		ffi.command_type_s2c = S2CCommandFFIType::Unload;
		ffi.object_id = self.global_object_id;
	}
}


impl Client2ServerFFIConverter for UnloadGameObjectCommand {
	fn from_ffi(ffi: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Unload);
		C2SCommandUnion::Unload(
			UnloadGameObjectCommand {
				global_object_id: ffi.object_id
			})
	}
}

