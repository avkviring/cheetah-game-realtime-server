use cheetah_relay_common::commands::command::load::CreateGameObjectCommand;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;

use crate::client::command::C2SCommandUnion;
use crate::client::ffi::{C2SCommandFFIType, Client2ServerFFIConverter, Command, S2CCommandFFIType, Server2ClientFFIConverter};
use crate::client::ffi::counters::Counters;
use crate::client::ffi::structures::Structures;

impl Server2ClientFFIConverter for CreateGameObjectCommand {
	fn to_ffi(self, command: &mut Command) {
		command.command_type_s2c = S2CCommandFFIType::Create;
		command.object_id.set_from(&self.object_id);
		command.object_template = self.template;
		command.access_group = self.access_groups.groups;
		command.long_counters = Counters::from(&self.fields.longs);
		command.float_counters = Counters::from(&self.fields.floats);
		command.structures = Structures::from(&self.fields.structures);
	}
}

impl Client2ServerFFIConverter for CreateGameObjectCommand {
	fn from_ffi(command: &Command) -> C2SCommandUnion {
		debug_assert!(command.command_type_c2s == C2SCommandFFIType::Create);
		let structures = From::from(&command.structures);
		C2SCommandUnion::Create(
			CreateGameObjectCommand {
				object_id: command.object_id.to_common_game_object_id(),
				template: command.object_template,
				access_groups: AccessGroups::from(command.access_group),
				fields: GameObjectFields {
					longs: From::from(&command.long_counters),
					floats: From::from(&command.float_counters),
					structures: structures,
				},
			})
	}
}


