use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::create::CreateGameObject;
use cheetah_common::commands::CommandTypeId;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn should_load_self_object_after_attach() {
	let mut helper = IntegrationTestHelper::new(Default::default());
	let (member_id, private_key) = helper.create_member();
	let client = helper.create_client(member_id, &private_key, 0);
	let object_id = helper.create_member_object(client);
	ffi::command::room::attach_to_room(client);
	helper.wait_udp();
	let commands = helper.receive(client);

	assert!(commands.len() > 0);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::CreateGameObject,
			command: S2CommandUnionFFI {
				create: CreateGameObject {
					object_id: object_id,
					template: IntegrationTestServerBuilder::DEFAULT_TEMPLATE,
					access_groups: IntegrationTestServerBuilder::DEFAULT_ACCESS_GROUP,
				}
			},
		}
	);
}
