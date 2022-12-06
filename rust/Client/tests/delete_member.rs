use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;
use cheetah_client::ffi::execute_with_client;
use cheetah_common::network::client::{ConnectionStatus, DisconnectedReason};
use cheetah_common::protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_common::protocol::others::member_id::MemberAndRoomId;
use std::thread;
use std::time::Duration;

pub mod helpers;

#[test]
fn should_disconnect_on_delete_member() {
	let builder = IntegrationTestServerBuilder::default();

	let mut helper = IntegrationTestHelper::new(builder);
	let (member_id, private_key) = helper.create_member();
	let client = helper.create_client(member_id, &private_key);
	helper.wait_udp();

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert_eq!(status, ConnectionStatus::Connected);
		Ok(())
	});

	assert!(
		helper
			.server
			.delete_member(MemberAndRoomId {
				room_id: helper.room_id,
				member_id,
			})
			.is_ok(),
		"want successful delete_member"
	);

	thread::sleep(Duration::from_millis(100));

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert!(matches!(
			status,
			ConnectionStatus::Disconnected(DisconnectedReason::ByCommand(DisconnectByCommandReason::MemberDeleted))
		));
		Ok(())
	});
}