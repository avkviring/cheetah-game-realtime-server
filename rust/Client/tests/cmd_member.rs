use cheetah_client::ffi;
use cheetah_client::ffi::command::{S2CCommandFFI, S2CommandUnionFFI};
use cheetah_common::commands::types::member::MemberDisconnected;
use cheetah_common::commands::CommandTypeId;

use crate::helpers::helper::setup;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn should_disconnect() {
	let (helper, [client1, client2]) = setup(IntegrationTestServerBuilder::default());
	ffi::command::room::attach_to_room(client1);
	ffi::command::room::attach_to_room(client2);
	helper.receive(client1);
	helper.receive(client2);
	helper.wait_udp();
	ffi::client::destroy_client(client2);
	helper.wait_udp();
	let commands = helper.receive(client1);
	assert_eq!(
		commands[0],
		S2CCommandFFI {
			command_type: CommandTypeId::MemberDisconnected,
			command: S2CommandUnionFFI {
				member_disconnect: MemberDisconnected { member_id: 2 }
			}
		}
	);
}
