use cheetah_client::ffi;
use cheetah_client::ffi::command::{BinaryFieldFFI, BufferFFI, S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::object::GameObjectId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn test() {
	let (helper, [client1, client2]) = setup(IntegrationTestServerBuilder::default());
	ffi::command::room::attach_to_room(client2);
	let mut object_id = GameObjectId::default();
	ffi::command::object::create_object(client1, 1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	ffi::command::object::created_object(client1, &object_id, false, &Default::default());
	helper.receive(client2);

	let mut event_buffer = BufferFFI { len: 1, ..Default::default() };
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;
	ffi::command::event::send_event(client1, &object_id, event_field_id, &event_buffer);

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SendEvent,
			command: S2CommandUnionFFI {
				buffer_field: BinaryFieldFFI {
					object_id,
					field_id: event_field_id,
					value: event_buffer.into(),
				}
			}
		}
	);
}
