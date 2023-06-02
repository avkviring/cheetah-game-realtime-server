use std::thread;
use std::time::Duration;

use cheetah_client::ffi::execute_with_client;
use cheetah_common::network::ConnectionStatus;
use cheetah_protocol::disconnect::command::DisconnectByCommandReason;
use cheetah_protocol::frame::disconnected_reason::DisconnectedReason;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;

pub mod helpers;

#[test]
fn should_disconnect_on_delete_room() {
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

	assert!(helper.server.delete_room(helper.room_id).is_ok(), "want successful delete_room");

	thread::sleep(Duration::from_millis(100));

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert!(matches!(status, ConnectionStatus::Disconnected(DisconnectedReason::ByCommand(DisconnectByCommandReason::RoomDeleted))));
		Ok(())
	});
}
