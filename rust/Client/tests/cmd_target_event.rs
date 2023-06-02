use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::event::EventCommand;
use cheetah_common::commands::CommandTypeId;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::object::GameObjectId;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn test() {
	let builder = IntegrationTestServerBuilder::default();
	let mut helper = IntegrationTestHelper::new(builder);
	let (member1, member1_key) = helper.create_member();
	let (member2, member2_key) = helper.create_member();
	let (member3, member3_key) = helper.create_member();
	let client1 = helper.create_client(member1, &member1_key);
	let client2 = helper.create_client(member2, &member2_key);
	let client3 = helper.create_client(member3, &member3_key);

	ffi::command::room::attach_to_room(client2);
	ffi::command::room::attach_to_room(client3);

	let mut object_id = GameObjectId::default();
	ffi::command::object::create_object(client1, 1, IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP.0, &mut object_id);
	ffi::command::object::created_object(client1, &object_id, false, &Buffer::default());
	helper.receive(client2);
	helper.receive(client3);

	let mut event_buffer = Buffer { len: 1, ..Default::default() };
	event_buffer.buffer[0] = 100;
	let event_field_id = 10;

	ffi::command::event::send_target_event(client1, member2, &object_id, event_field_id, &event_buffer);

	let commands = helper.receive(client3);
	assert_eq!(commands, vec![]);

	let commands = helper.receive(client2);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::SendEvent,
			command: S2CommandUnionFFI {
				event: EventCommand {
					object_id,
					field_id: event_field_id,
					event: event_buffer,
				}
			}
		}
	);
}
