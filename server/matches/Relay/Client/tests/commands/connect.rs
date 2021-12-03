use std::thread;
use std::time::Duration;

use cheetah_matches_relay_client::ffi::execute_with_client;
use cheetah_matches_relay_common::network::client::ConnectionStatus;
use cheetah_matches_relay_common::protocol::disconnect::watcher::DisconnectWatcher;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn should_connect_to_server() {
	let builder = IntegrationTestServerBuilder::default();
	let mut helper = IntegrationTestHelper::new(builder);
	let (user_id, user_key) = helper.create_user();
	let client = helper.create_client(user_id, user_key);
	helper.wait_udp();
	execute_with_client(client, |api| {
		assert_eq!(api.get_connection_status(), ConnectionStatus::Connected);
	})
	.unwrap();
}

#[test]
fn should_disconnect_when_server_closed() {
	let builder = IntegrationTestServerBuilder::default();

	let mut helper = IntegrationTestHelper::new(builder);
	let (user_id, user_key) = helper.create_user();
	let client = helper.create_client(user_id, user_key);
	helper.wait_udp();

	execute_with_client(client, |api| {
		assert_eq!(api.get_connection_status(), ConnectionStatus::Connected);
	})
	.unwrap();

	drop(helper);

	execute_with_client(client, |api| {
		api.set_protocol_time_offset(DisconnectWatcher::TIMEOUT);
	})
	.unwrap();
	thread::sleep(Duration::from_millis(100));

	execute_with_client(client, |api| {
		assert_eq!(api.get_connection_status(), ConnectionStatus::Disconnected);
	})
	.unwrap();
}
