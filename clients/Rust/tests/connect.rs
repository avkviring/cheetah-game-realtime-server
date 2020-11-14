use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use cheetah_relay_client::do_get_connection_status;
use cheetah_relay_common::protocol::disconnect::watcher::DisconnectWatcher;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::udp::client::ConnectionStatus;

use crate::helpers::Helper;

pub mod helpers;

#[test]
fn should_connect_to_server() {
	let mut helper = Helper::new();
	let (server, client) = helper.setup_server_and_client();
	helper.wait_first_frame();
	
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, ConnectionStatus::Connected); },
		|| { assert!(false) },
	);
	drop(server)
}

#[test]
fn should_disconnect_when_server_closed() {
	let mut helper = Helper::new();
	let (server, client) = helper.setup_server_and_client();
	helper.wait_first_frame();
	
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, ConnectionStatus::Connected); },
		|| { assert!(false) },
	);
	
	drop(server);
	
	helper.set_protocol_time_offset(client, DisconnectWatcher::TIMEOUT);
	thread::sleep(Duration::from_millis(100));
	
	do_get_connection_status(
		client,
		|status| { assert_eq!(status, ConnectionStatus::Disconnected); },
		|| { assert!(false) },
	);
}
