use cheetah_relay_client::client::command::C2SCommandUnion;
use cheetah_relay_client::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::network::command::float_counter::{IncrementFloatCounterC2SCommand, SetFloatCounterCommand};
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;

#[test]
fn should_to_ffi() {
	let object_id = ClientGameObjectId::new(100, ClientOwner::Root);
	let command = SetFloatCounterCommand {
		object_id: object_id.clone(),
		field_id: 10,
		value: 1.0,
	};
	
	let mut ffi = CommandFFI::default();
	command.to_ffi(&mut ffi);
	
	assert_eq!(S2CCommandFFIType::SetFloatCounter, ffi.command_type_s2c);
	assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
	assert_eq!(10, ffi.field_id);
	assert_eq!(1, ffi.float_value as u8);
}

#[test]
fn should_set_float_counter_from_ffi() {
	let object_id = ClientGameObjectId::new(100, ClientOwner::Root);
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::SetFloatCounter;
	ffi.object_id.set_from(&object_id);
	ffi.field_id = 10;
	ffi.float_value = 1.0;
	let command = SetFloatCounterCommand::from_ffi(&ffi);
	assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.object_id == object_id));
	assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.field_id == 10));
	assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.value as u8 == 1));
}

#[test]
fn should_increment_float_counter_from_ffi() {
	let object_id = ClientGameObjectId::new(100, ClientOwner::Root);
	let mut ffi = CommandFFI::default();
	ffi.command_type_c2s = C2SCommandFFIType::IncrementFloatCounter;
	ffi.object_id.set_from(&object_id);
	ffi.field_id = 10;
	ffi.float_value = 1.0;
	let command = IncrementFloatCounterC2SCommand::from_ffi(&ffi);
	assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.object_id == object_id));
	assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.field_id == 10));
	assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.increment as u8 == 1));
}