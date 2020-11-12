use cheetah_relay_common::commands::command::long_counter::{IncrementLongC2SCommand, SetLongCommand};

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::commands::command::C2SCommandUnion;

impl Server2ClientFFIConverter for SetLongCommand {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::SetLongCounter;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.long_value = self.value;
	}
}

impl Client2ServerFFIConverter for SetLongCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::SetLongCounter);
		C2SCommandUnion::SetLongCounter(
			SetLongCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				value: ffi.long_value,
			})
	}
}

impl Client2ServerFFIConverter for IncrementLongC2SCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::IncrementLongCounter);
		C2SCommandUnion::IncrementLongCounter(
			IncrementLongC2SCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				increment: ffi.long_value,
			})
	}
}

#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	use cheetah_relay_common::commands::command::long_counter::{SetLongCommand, IncrementLongC2SCommand};
	use crate::client::ffi::{Command, S2CCommandFFIType, C2SCommandFFIType, Client2ServerFFIConverter, Server2ClientFFIConverter};
	use cheetah_relay_common::commands::command::C2SCommandUnion;
	
	#[test]
	fn should_to_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 10,
			value: 1,
		};
		
		let mut ffi = Command::default();
		command.to_ffi(&mut ffi);
		
		assert_eq!(S2CCommandFFIType::SetLongCounter, ffi.command_type_s2c);
		assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
		assert_eq!(10, ffi.field_id);
		assert_eq!(1, ffi.long_value as u8);
	}
	
	#[test]
	fn should_set_float_counter_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::SetLongCounter;
		ffi.object_id.set_from(&object_id);
		ffi.field_id = 10;
		ffi.long_value = 1;
		let command = SetLongCommand::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::SetLongCounter(ref long_counter) if long_counter.object_id == object_id));
		assert!(matches!(&command,C2SCommandUnion::SetLongCounter(ref long_counter) if long_counter.field_id == 10));
		assert!(matches!(&command,C2SCommandUnion::SetLongCounter(ref long_counter) if long_counter.value == 1));
	}
	
	#[test]
	fn should_increment_float_counter_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::IncrementLongCounter;
		ffi.object_id.set_from(&object_id);
		ffi.field_id = 10;
		ffi.long_value = 1;
		let command = IncrementLongC2SCommand::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::IncrementLongCounter(ref long_counter) if long_counter.object_id == object_id));
		assert!(matches!(&command,C2SCommandUnion::IncrementLongCounter(ref long_counter) if long_counter.field_id == 10));
		assert!(matches!(&command,C2SCommandUnion::IncrementLongCounter(ref long_counter) if long_counter.increment == 1));
	}
}