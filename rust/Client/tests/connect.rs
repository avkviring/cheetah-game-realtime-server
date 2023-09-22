use cheetah_game_realtime_protocol::frame::disconnected_reason::DisconnectedReason;
use std::thread;
use std::time::Duration;

use crate::helpers::helper::IntegrationTestHelper;
use crate::helpers::server::IntegrationTestServerBuilder;
use cheetah_client::ffi::execute_with_client;
use cheetah_common::network::ConnectionStatus;

pub mod helpers;

#[test]
fn should_connect_to_server() {
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
}

#[test]
fn should_disconnect_when_server_closed() {
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

	drop(helper);

	execute_with_client(client, |api| {
		api.set_protocol_time_offset(IntegrationTestServerBuilder::DISCONNECT_DURATION).unwrap();
		Ok(())
	});
	thread::sleep(Duration::from_millis(100));

	execute_with_client(client, |api| {
		let status = api.get_connection_status().unwrap();
		assert!(matches!(status, ConnectionStatus::Disconnected(DisconnectedReason::Timeout)));
		Ok(())
	});
}
