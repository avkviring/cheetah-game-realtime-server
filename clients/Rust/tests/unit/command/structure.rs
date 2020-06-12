use cheetah_relay_client::client::command::C2SCommandUnion;
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, FieldFFIBinary, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::network::command::structure::StructureCommand;

#[test]
fn should_to_ffi() {
	let command = StructureCommand {
		global_object_id: 100,
		field_id: 10,
		structure: vec![1, 2, 3, 4, 5],
	};
	
	let mut ffi = CommandFFI::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::Structure, ffi.command_type_s2c);
	assert_eq!(100, ffi.object_id);
	assert_eq!(vec![1 as u8, 2, 3, 4, 5].as_slice(), ffi.structure.as_slice())
}

#[test]
fn should_from_ffi() {
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::Structure;
	ffi.object_id = 100;
	ffi.field_id = 10;
	ffi.structure = FieldFFIBinary::from(vec![1, 2, 3]);
	let command = StructureCommand::from_ffi(&ffi);
	
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.global_object_id == 100));
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.field_id == 10));
	assert!(matches!(&command,C2SCommandUnion::Structure(ref structure) if structure.structure.as_slice() == vec![1,2,3].as_slice()));
}