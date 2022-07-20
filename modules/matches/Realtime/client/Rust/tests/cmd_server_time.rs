use std::thread;
use std::time::Duration;

use cheetah_matches_realtime_client::ffi;

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
fn test() {
	let (_helper, [client1]) = setup(Default::default());
	ffi::command::room::attach_to_room(client1);
	thread::sleep(Duration::from_millis(2000));
	let mut time: u64 = 0;
	ffi::client::get_server_time(client1, &mut time);
	assert!(time >= 1000);
}
