use cheetah_relay_common::commands::command::event::EventCommand;

use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use cheetah_relay_common::commands::command::C2SCommandUnion;

impl Server2ClientFFIConverter for EventCommand {
	fn to_ffi(self, ffi: &mut Command) {
		ffi.command_type_s2c = S2CCommandFFIType::Event;
		ffi.object_id.set_from(&self.object_id);
		ffi.field_id = self.field_id;
		ffi.event = From::from(self.event);
	}
}

impl Client2ServerFFIConverter for EventCommand {
	fn from_ffi(ffi: &Command) -> C2SCommandUnion {
		debug_assert!(ffi.command_type_c2s == C2SCommandFFIType::Event);
		C2SCommandUnion::Event(
			EventCommand {
				object_id: ffi.object_id.to_common_game_object_id(),
				field_id: ffi.field_id,
				event: From::from(ffi.event),
			})
	}
}


#[cfg(test)]
mod tests {
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::commands::command::event::EventCommand;
	use crate::client::ffi::{Server2ClientFFIConverter, S2CCommandFFIType, Command, C2SCommandFFIType, Client2ServerFFIConverter};
	use cheetah_relay_common::room::owner::ClientOwner;
	use cheetah_relay_common::commands::command::C2SCommandUnion;
	use crate::client::ffi::bytes::Bytes;
	
	#[test]
	fn should_to_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		let command = EventCommand {
			object_id: object_id.clone(),
			field_id: 10,
			event: vec![1, 2, 3, 4, 5],
		};
		
		let mut ffi = Command::default();
		command.to_ffi(&mut ffi);
		
		assert_eq!(S2CCommandFFIType::Event, ffi.command_type_s2c);
		assert_eq!(object_id, ffi.object_id.to_common_game_object_id());
		assert_eq!(vec![1 as u8, 2, 3, 4, 5].as_slice(), ffi.event.as_slice())
	}
	
	#[test]
	fn should_from_ffi() {
		let object_id = GameObjectId::new(100, ClientOwner::Root);
		
		let mut ffi = Command::default();
		ffi.command_type_c2s = C2SCommandFFIType::Event;
		ffi.object_id.set_from(&object_id);
		ffi.field_id = 10;
		ffi.event = Bytes::from(vec![1, 2, 3]);
		let command = EventCommand::from_ffi(&ffi);
		assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.object_id == object_id));
		assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.field_id == 10));
		assert!(matches!(&command,C2SCommandUnion::Event(ref event) if event.event.as_slice() == vec![1,2,3].as_slice()));
	}
}