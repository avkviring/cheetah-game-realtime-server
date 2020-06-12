use cheetah_relay_common::network::command::upload::{UploadGameObjectC2SCommand, UploadGameObjectS2CCommand};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, FieldsFFI, Client2ServerFFIConverter, CommandFFI, S2CCommandFFIType, Server2ClientFFIConverter};

impl Server2ClientFFIConverter for UploadGameObjectS2CCommand {
	fn to_ffi(self, command: &mut CommandFFI) {
		command.command_type_s2c = S2CCommandFFIType::Upload;
		command.object_id = self.global_object_id;
		command.long_counters = FieldsFFI::from(&self.fields.long_counters);
		command.float_counters = FieldsFFI::from(&self.fields.float_counters);
		command.structures = FieldsFFI::from(&self.fields.structures);
	}
}

impl Client2ServerFFIConverter for UploadGameObjectC2SCommand {
	fn from_ffi(command: &CommandFFI) -> C2SCommandUnion {
		debug_assert!(command.command_type_c2s == C2SCommandFFIType::Upload);
		C2SCommandUnion::Upload(
			UploadGameObjectC2SCommand {
				local_id: command.object_id as u32,
				access_groups: AccessGroups::from(command.access_group),
				fields: GameObjectFields {
					long_counters: From::from(command.long_counters),
					float_counters: From::from(command.float_counters),
					structures: From::from(command.structures),
				},
			})
	}
}


