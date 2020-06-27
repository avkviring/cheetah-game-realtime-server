use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use crate::client::ffi::counters::Counters;
use crate::client::ffi::structures::Structures;

impl Server2ClientFFIConverter for UploadGameObjectCommand {
	fn to_ffi(self, command: &mut Command) {
		command.command_type_s2c = S2CCommandFFIType::Upload;
		command.object_id.set_from(&self.object_id);
		command.access_group = self.access_groups.groups;
		command.long_counters = Counters::from(&self.fields.long_counters);
		command.float_counters = Counters::from(&self.fields.float_counters);
		command.structures = Structures::from(&self.fields.structures);
	}
}

impl Client2ServerFFIConverter for UploadGameObjectCommand {
	fn from_ffi(command: &Command) -> C2SCommandUnion {
		debug_assert!(command.command_type_c2s == C2SCommandFFIType::Upload);
		let structures = From::from(&command.structures);
		C2SCommandUnion::Upload(
			UploadGameObjectCommand {
				object_id: command.object_id.to_common_game_object_id(),
				access_groups: AccessGroups::from(command.access_group),
				fields: GameObjectFields {
					long_counters: From::from(&command.long_counters),
					float_counters: From::from(&command.float_counters),
					structures: structures,
				},
			})
	}
}


