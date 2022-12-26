use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::binary_value::Buffer;
use cheetah_common::commands::types::structure::SetStructureCommand;
use cheetah_common::commands::CommandTypeId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_set() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client2);

	let structure_buffer = Buffer::from(vec![100].as_slice());
	let structure_field_id = 10;
	ffi::command::structure::set_structure(client1, &object_id, structure_field_id, &structure_buffer);

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SetStructure,
			command: S2CommandUnionFFI {
				set_structure: SetStructureCommand {
					object_id,
					field_id: structure_field_id,
					value: structure_buffer,
				}
			}
		}
	);
}
