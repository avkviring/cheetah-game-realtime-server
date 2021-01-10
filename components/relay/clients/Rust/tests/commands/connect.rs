use std::thread;
use std::time::Duration;

use cheetah_relay_client::ffi::client::set_current_client;
use cheetah_relay_client::ffi::execute_with_client;
use cheetah_relay_common::network::client::ConnectionStatus;
use cheetah_relay_common::protocol::disconnect::watcher::DisconnectWatcher;

use crate::helpers::helper::*;
use crate::helpers::server::*;

#[test]
fn should_connect_to_server() {
	let mut builder = IntegrationTestServerBuilder::default();
	let (user_id, user_key) = builder.make_user_key();
	let helper = IntegrationTestHelper::new(builder);
	let _ = helper.create_client(user_id, user_key);
	helper.wait_udp();
	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Connected)).unwrap();
}

#[test]
fn should_disconnect_when_server_closed() {
	let mut builder = IntegrationTestServerBuilder::default();
	let (user_id, user_key) = builder.make_user_key();

	let helper = IntegrationTestHelper::new(builder);
	let client = helper.create_client(user_id, user_key);
	helper.wait_udp();

	set_current_client(client);
	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Connected)).unwrap();

	drop(helper);

	set_current_client(client);
	execute_with_client(|api| {
		api.set_protocol_time_offset(DisconnectWatcher::TIMEOUT);
	})
	.unwrap();
	thread::sleep(Duration::from_millis(100));

	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Disconnected)).unwrap();
}