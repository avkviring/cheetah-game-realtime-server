use cheetah_relay_common::commands::command::long::SetLongCommand;
use cheetah_relay_common::commands::command::C2SCommand;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use std::thread;

use crate::helper::TestEnvBuilder;
use std::time::Duration;

pub mod helper;

///
/// Тестируем работу сервера под большой нагрузкой
///
#[test]
pub fn stress_test() {
	let mut builder = TestEnvBuilder::default();
	let user_id_1 = 1;
	let user_id_2 = 2;
	builder.create_user(user_id_1);
	builder.create_user(user_id_2);

	let game_object_id = 1;
	builder.create_object(user_id_1, game_object_id);
	let mut env = builder.build();

	env.connect(user_id_1, TestEnvBuilder::ROOM_ID);
	env.connect(user_id_2, TestEnvBuilder::ROOM_ID);

	env.send_to_server(user_id_2, C2SCommand::AttachToRoom);
	env.cycle();
	thread::sleep(Duration::from_millis(10));

	let count = 500;
	for i in 0..count {
		let command = SetLongCommand {
			object_id: GameObjectId {
				owner: ObjectOwner::User(user_id_1),
				id: game_object_id,
			},
			field_id: 1,
			value: i,
		};
		env.send_to_server(user_id_1, C2SCommand::SetLong(command));
		env.cycle();
	}

	thread::sleep(Duration::from_millis(10));
	env.cycle();

	let in_commands = env.get_input_commands(user_id_2);
	assert_eq!(in_commands.len(), count as usize + 2); // +2 - команда создания объекта
}
