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

#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
	use cheetah_relay_common::room::object::GameObjectId;
	use crate::client::ffi::{Command, Server2ClientFFIConverter, S2CCommandFFIType, C2SCommandFFIType, Client2ServerFFIConverter};
	use cheetah_relay_common::room::owner::ClientOwner;
	use cheetah_relay_common::commands::command::C2SCommandUnion;
	
	#[test]
	fn should_to_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let command = DeleteGameObjectCommand {
			object_id: object_id.clone(),
		};
		
		let mut ffi = Command::default();
		command.to_ffi(&mut ffi);
		
		assert_eq!(S2CCommandFFIType::Unload, ffi.command_type_s2c);
		assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
	}
	
	#[test]
	fn should_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::Unload;
		ffi.object_id.set_from(&object_id);
		let command = DeleteGameObjectCommand::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::Delete(ref unload) if unload.object_id == object_id));
	}
}

