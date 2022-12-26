use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::float::SetDoubleCommand;
use cheetah_common::commands::CommandTypeId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_inc() {
	let (helper, [client1, client2]) = setup(Default::default());
	let object_id = helper.create_member_object(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client2);

	ffi::command::float_value::inc_double_value(client1, &object_id, 1, 100.0);
	ffi::command::float_value::inc_double_value(client1, &object_id, 1, 100.0);

	let commands = helper.receive(client2);

	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SetDouble,
			command: S2CommandUnionFFI {
				set_double: SetDoubleCommand { object_id, field_id: 1, value: 100.0 }
			}
		}
	);

	assert_eq!(
		commands[1],
		S2CCommandFFI {
			command_type: CommandTypeId::SetDouble,
			command: S2CommandUnionFFI {
				set_double: SetDoubleCommand { object_id, field_id: 1, value: 200.0 }
			}
		}
	);
}

#[test]
fn should_set() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client2);

	ffi::command::float_value::set_double_value(client1, &object_id, 1, 100.0);
	ffi::command::float_value::set_double_value(client1, &object_id, 1, 200.0);

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SetDouble,
			command: S2CommandUnionFFI {
				set_double: SetDoubleCommand { object_id, field_id: 1, value: 100.0 }
			}
		}
	);

	assert_eq!(
		commands[1],
		S2CCommandFFI {
			command_type: CommandTypeId::SetDouble,
			command: S2CommandUnionFFI {
				set_double: SetDoubleCommand { object_id, field_id: 1, value: 200.0 }
			}
		}
	);
}
