use cheetah_relay_common::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use cheetah_relay_common::commands::command::long::SetLongCommand;
use cheetah_relay_common::commands::command::{C2SCommand, S2CCommand};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;

use crate::helper::*;

pub mod helper;

#[test]
pub fn test_create_object() {
	let mut builder = TestEnvBuilder::default();
	let user_id_1 = 1;
	let user_id_2 = 2;
	let user_id_3 = 3;
	builder.create_user(user_id_1);
	builder.create_user(user_id_2);
	builder.create_user(user_id_3);

	let mut env = builder.build();
	env.connect(user_id_1);
	env.connect(user_id_2);

	env.send_to_server(user_id_2, C2SCommand::AttachToRoom);

	let object_id = GameObjectId::new(1, ObjectOwner::User(user_id_1));
	let create_command = CreateGameObjectCommand {
		object_id: object_id.clone(),
		template: 0,
		access_groups: TestEnv::DEFAULT_ACCESS_GROUP,
	};
	env.send_to_server(user_id_1, C2SCommand::Create(create_command));
	env.send_to_server(
		user_id_1,
		C2SCommand::SetLong(SetLongCommand {
			object_id: object_id.clone(),
			field_id: 1,
			value: 100,
		}),
	);
	env.cycle();

	// проверяем что команд по объекту не пришло
	let commands = env.get_input_commands(user_id_2);
	assert!(commands.is_empty());

	// проверяем что дошла команда подтверждения создания объекта
	env.send_to_server(
		user_id_1,
		C2SCommand::Created(CreatedGameObjectCommand {
			object_id: object_id.clone(),
		}),
	);
	env.cycle();

	let mut commands = env.get_input_commands(user_id_2);
	assert!(matches!(commands.remove(0), S2CCommand::Create(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::Created(c) if c.object_id == object_id));

	// проверяем загрузку уже созданного объекта
	env.connect(user_id_3);
	env.send_to_server(user_id_3, C2SCommand::AttachToRoom);
	env.cycle();

	let mut commands = env.get_input_commands(user_id_3);
	assert!(matches!(commands.remove(0), S2CCommand::Create(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::Created(c) if c.object_id == object_id));
}
