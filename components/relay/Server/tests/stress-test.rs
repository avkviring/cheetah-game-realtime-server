use std::thread;
use std::time::Duration;

use cheetah_relay::test_env::IntegrationTestServerBuider;
use cheetah_relay_common::commands::command::long::SetLongCommand;
use cheetah_relay_common::commands::command::C2SCommand;

use crate::helper::*;

pub mod helper;

///
/// Тестируем работу сервера под большой нагрузкой
///
#[test]
pub fn stress_test() {
	let mut builder = IntegrationTestServerBuider::default();
	let (user1_id, user1_key) = builder.create_user();
	let (user2_id, user2_key) = builder.create_user();
	let object_id = builder.create_object(user1_id, 0);

	let mut helper = IntegrationTestHelper::new(builder);

	helper.connect(user1_id, user1_key);
	helper.connect(user2_id, user2_key);

	helper.send_to_server(user2_id, C2SCommand::AttachToRoom);
	helper.cycle();
	thread::sleep(Duration::from_millis(10));

	let count = 500;
	for i in 0..count {
		let command = SetLongCommand {
			object_id: object_id.clone(),
			field_id: 1,
			value: i,
		};
		helper.send_to_server(user1_id, C2SCommand::SetLong(command));
		helper.cycle();
	}

	thread::sleep(Duration::from_millis(10));
	helper.cycle();

	let in_commands = helper.get_input_commands(user2_id);
	assert_eq!(in_commands.len(), count as usize + 2); // +2 - команда создания объекта
}
