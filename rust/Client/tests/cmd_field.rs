use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::field::DeleteField;
use cheetah_common::commands::types::float::DoubleField;
use cheetah_common::commands::types::long::LongField;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::field::FieldType;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_delete_field_ffi() {
	let (helper, [client1, client2]) = setup(Default::default());

	let object_id = helper.create_member_object(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client2);

	ffi::command::field::delete_field(client1, &object_id, 1, FieldType::Long);

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::DeleteField,
			command: S2CommandUnionFFI {
				delete_field: DeleteField {
					field_id: 1,
					object_id,
					field_type: FieldType::Long,
				}
			}
		}
	);
}

#[test]
fn should_allow_fields_with_different_types_but_same_id() {
	let (helper, [client1, client2]) = setup(Default::default());
	ffi::command::room::attach_to_room(client2);
	let object_id = helper.create_member_object(client1);
	helper.receive(client2);

	ffi::command::float_value::set_double_value(client1, &object_id, 1, 100.0);
	ffi::command::long_value::set_long_value(client1, &object_id, 1, 50);

	let commands = helper.receive(client2);

	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SetDouble,
			command: S2CommandUnionFFI {
				set_double: DoubleField { object_id, field_id: 1, value: 100.0 }
			}
		}
	);

	assert_eq!(
		commands[1],
		S2CCommandFFI {
			command_type: CommandTypeId::SetLong,
			command: S2CommandUnionFFI {
				set_long: LongField { object_id, field_id: 1, value: 50 }
			}
		}
	);
}
