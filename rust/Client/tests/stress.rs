use std::thread;
use std::time::Duration;

use cheetah_client::ffi;
use cheetah_client::ffi::logs::{init_logger, set_max_log_level, LogLevel};

use crate::helpers::helper::setup;

pub mod helpers;

#[test]
pub fn stress_create_lot_of_objects_test() {
	const COUNT_OBJECTS: usize = 100;
	init_logger();
	set_max_log_level(LogLevel::Warn);
	let (helper, [client1, client2]) = setup(Default::default());
	ffi::command::room::attach_to_room(client1);
	ffi::command::room::attach_to_room(client2);
	thread::sleep(Duration::from_secs(1));
	for _ in 0..COUNT_OBJECTS {
		helper.create_member_object(client1);
	}
	thread::sleep(Duration::from_secs(1));
	let commands = helper.receive(client2);
	assert_eq!(commands.len(), COUNT_OBJECTS * 2)
}
