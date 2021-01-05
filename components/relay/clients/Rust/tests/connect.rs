use cheetah_relay_client::ffi::client::set_current_client;
use cheetah_relay_client::ffi::execute_with_client;
use cheetah_relay_common::network::client::ConnectionStatus;
use cheetah_relay_common::protocol::disconnect::watcher::DisconnectWatcher;
use std::thread;
use std::time::Duration;

use crate::helpers::Helper;

pub mod helpers;

#[test]
fn should_connect_to_server() {
	let mut helper = Helper::new();
	let (server, _) = helper.setup_server_and_client();
	helper.wait_first_frame();

	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Connected)).unwrap();
	drop(server)
}

#[test]
fn should_disconnect_when_server_closed() {
	let mut helper = Helper::new();
	let (server, client) = helper.setup_server_and_client();
	helper.wait_first_frame();

	set_current_client(client);
	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Connected)).unwrap();

	drop(server);

	set_current_client(client);
	execute_with_client(|api| {
		api.set_protocol_time_offset(DisconnectWatcher::TIMEOUT);
	})
	.unwrap();
	thread::sleep(Duration::from_millis(100));

	execute_with_client(|api| assert_eq!(api.get_connection_status(), ConnectionStatus::Disconnected)).unwrap();
}
