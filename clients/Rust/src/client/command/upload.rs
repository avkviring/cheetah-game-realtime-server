use cheetah_relay_common::network::command::upload::UploadGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, CommandFFI, FieldsFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for UploadGameObjectCommand {
	fn to_ffi(self, command: &mut CommandFFI) {
		command.command_type_s2c = S2CCommandFFIType::Upload;
		command.object_id.set_from(&self.object_id);
		command.access_group = self.access_groups.groups;
		command.long_counters = FieldsFFI::from(&self.fields.long_counters);
		command.float_counters = FieldsFFI::from(&self.fields.float_counters);
		command.structures = FieldsFFI::from(&self.fields.structures);
	}
}

impl Client2ServerFFIConverter for UploadGameObjectCommand {
	fn from_ffi(command: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(command.command_type_c2s == C2SCommandFFIType::Upload);
		C2SCommandUnion::Upload(
			UploadGameObjectCommand {
				object_id: command.object_id.to_common_game_object_id(),
				access_groups: AccessGroups::from(command.access_group),
				fields: GameObjectFields {
					long_counters: From::from(command.long_counters),
					float_counters: From::from(command.float_counters),
					structures: From::from(command.structures),
				},
			})
	}
}


