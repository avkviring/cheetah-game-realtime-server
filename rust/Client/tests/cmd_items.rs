use std::thread;
use std::time::Duration;

use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::structure::BinaryField;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::field::FieldId;
use cheetah_common::room::object::GameObjectId;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn should_items_loaded() {
	let (helper, [client1, client2]) = setup(Default::default());
	let object_id = helper.create_member_object(client1);

	let (field_id, item_1, item_2) = add_items(client1, &object_id);
	thread::sleep(Duration::from_millis(200));

	ffi::command::room::attach_to_room(client2);
	let commands = helper.receive(client2);
	assert_eq!(
		commands[1],
		S2CCommandFFI {
			command_type: CommandTypeId::AddItem,
			command: S2CommandUnionFFI {
				buffer_field: BinaryField { object_id, field_id, value: item_1 }
			}
		}
	);
	assert_eq!(
		commands[2],
		S2CCommandFFI {
			command_type: CommandTypeId::AddItem,
			command: S2CommandUnionFFI {
				buffer_field: BinaryField { object_id, field_id, value: item_2 }
			}
		}
	);
}

#[test]
fn should_set_items() {
	let (helper, [client1, client2]) = setup(Default::default());
	let object_id = helper.create_member_object(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client2);

	let (field_id, item_1, item_2) = add_items(client1, &object_id);
	thread::sleep(Duration::from_millis(200));

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::AddItem,
			command: S2CommandUnionFFI {
				buffer_field: BinaryField { object_id, field_id, value: item_1 }
			}
		}
	);
	assert_eq!(
		commands[1],
		S2CCommandFFI {
			command_type: CommandTypeId::AddItem,
			command: S2CommandUnionFFI {
				buffer_field: BinaryField { object_id, field_id, value: item_2 }
			}
		}
	);
}

fn add_items(client1: u16, object_id: &GameObjectId) -> (FieldId, Buffer, Buffer) {
	let field_id = 10;
	let item_1 = Buffer::from([100].as_slice());
	let item_2 = Buffer::from([200].as_slice());
	ffi::command::items::add_item(client1, &object_id, field_id, &item_1);
	ffi::command::items::add_item(client1, &object_id, field_id, &item_2);
	(field_id, item_1, item_2)
}
