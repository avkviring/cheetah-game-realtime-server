use cheetah_relay_client::client::command::C2SCommandUnion;
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, FieldFFIBinary, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::network::command::event::EventCommand;

#[test]
fn should_to_ffi() {
	let command = EventCommand {
		global_object_id: 100,
		field_id: 10,
		event: vec![1, 2, 3, 4, 5],
	};
	
	let mut ffi = CommandFFI::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::Event, ffi.command_type_s2c);
	assert_eq!(100, ffi.object_id);
	assert_eq!(vec![1 as u8, 2, 3, 4, 5].as_slice(), ffi.event.as_slice())
}

#[test]
fn should_from_ffi() {
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::Event;
	ffi.object_id = 100;
	ffi.field_id = 10;
	ffi.event = FieldFFIBinary::from(vec![1, 2, 3]);
	let command = EventCommand::from_ffi(&ffi);
	
	assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.global_object_id == 100));
	assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.field_id == 10));
	assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.event.as_slice() == vec![1,2,3].as_slice()));
}