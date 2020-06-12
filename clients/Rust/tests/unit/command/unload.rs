use cheetah_relay_client::client::command::C2SCommandUnion;
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, FieldFFIBinary, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::network::command::unload::UnloadGameObjectCommand;

#[test]
fn should_to_ffi() {
	let command = UnloadGameObjectCommand {
		global_object_id: 100,
	};
	
	let mut ffi = CommandFFI::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::Unload, ffi.command_type_s2c);
	assert_eq!(100, ffi.object_id);
}

#[test]
fn should_from_ffi() {
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::Unload;
	ffi.object_id = 100;
	let command = UnloadGameObjectCommand::from_ffi(&ffi);
	assert!(matches!(&command,C2SCommandUnion::Unload(ref unload) if unload.global_object_id == 100));
}