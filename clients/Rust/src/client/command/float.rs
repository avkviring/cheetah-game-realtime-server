use cheetah_relay_common::commands::command::C2SCommandUnion;
use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for SetFloat64Command {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::SetFloatCounter;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.float_value = self.value;
	}
}

impl Client2ServerFFIConverter for SetFloat64Command {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetFloatCounter);
		C2SCommandUnion::SetFloatCounter(
			SetFloat64Command {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				value: ffi.float_value,
			})
	}
}

impl Client2ServerFFIConverter for IncrementFloat64C2SCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::IncrementFloatCounter);
		C2SCommandUnion::IncrementFloatCounter(
			IncrementFloat64C2SCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				increment: ffi.float_value,
			})
	}
}


#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::C2SCommandUnion;
	use cheetah_relay_common::commands::command::float_counter::{IncrementFloat64C2SCommand, SetFloat64Command};
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
	
	#[test]
	fn should_to_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let command = SetFloat64Command {
			object_id: object_id.clone(),
			field_id: 10,
			value: 1.0,
		};
		
		let mut ffi = Command::default();
		command.to_ffi(&mut ffi);
		
		assert_eq!(S2CCommandFFIType::SetFloatCounter, ffi.command_type_s2c);
		assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
		assert_eq!(10, ffi.field_id);
		assert_eq!(1, ffi.float_value as u8);
	}
	
	#[test]
	fn should_set_float_counter_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::SetFloatCounter;
		ffi.object_id.set_from(&object_id);
		ffi.field_id = 10;
		ffi.float_value = 1.0;
		let command = SetFloat64Command::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.object_id == object_id));
		assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.field_id == 10));
		assert!(matches!(&command,C2SCommandUnion::SetFloatCounter(ref float_counter) if float_counter.value as u8 == 1));
	}
	
	#[test]
	fn should_increment_float_counter_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::IncrementFloatCounter;
		ffi.object_id.set_from(&object_id);
		ffi.field_id = 10;
		ffi.float_value = 1.0;
		let command = IncrementFloat64C2SCommand::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.object_id == object_id));
		assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.field_id == 10));
		assert!(matches!(&command,C2SCommandUnion::IncrementFloatCounter(ref float_counter) if float_counter.increment as u8 == 1));
	}
}