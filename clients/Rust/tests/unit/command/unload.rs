use cheetah_relay_common::commands::command::unload::UnloadGameObjectCommand;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::commands::command::C2SCommandUnion;

#[test]
fn should_to_ffi() {
	let object_id = ClientGameObjectId::new(100, ClientOwner::Root);
	let command = UnloadGameObjectCommand {
		object_id: object_id.clone(),
	};
	
	let mut ffi = Command::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::Unload, ffi.command_type_s2c);
	assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
}

#[test]
fn should_from_ffi() {
	let object_id = ClientGameObjectId::new(100, ClientOwner::Root);
	let mut ffi = Command::default();
	ffi.command_type_c2s = C2SCommandFFIType::Unload;
	ffi.object_id.set_from(&object_id);
	let command = UnloadGameObjectCommand::from_ffi(&ffi);
	assert!(matches!(&command,C2SCommandUnion::Unload(ref unload) if unload.object_id == object_id));
}