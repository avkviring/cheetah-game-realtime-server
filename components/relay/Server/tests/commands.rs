use cheetah_relay::test_env::IntegrationTestServerBuider;
use cheetah_relay_common::commands::command::load::{CreateGameObjectCommand, CreatedGameObjectCommand};
use cheetah_relay_common::commands::command::long::SetLongCommand;
use cheetah_relay_common::commands::command::{C2SCommand, S2CCommand};
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;

use crate::helper::*;

pub mod helper;

#[test]
pub fn test_create_object() {
	let mut builder = IntegrationTestServerBuider::default();
	let (user1_id, user1_key) = builder.create_user();
	let (user2_id, user2_key) = builder.create_user();
	let (user3_id, user3_key) = builder.create_user();

	let mut env = IntegrationTestHelper::new(builder);
	env.connect(user1_id, user1_key);
	env.connect(user2_id, user2_key);
	env.send_to_server(user2_id, C2SCommand::AttachToRoom);

	let object_id = GameObjectId::new(1, ObjectOwner::User(user1_id));
	let create_command = CreateGameObjectCommand {
		object_id: object_id.clone(),
		template: 0,
		access_groups: IntegrationTestHelper::DEFAULT_ACCESS_GROUP,
	};
	env.send_to_server(user1_id, C2SCommand::Create(create_command));
	env.send_to_server(
		user1_id,
		C2SCommand::SetLong(SetLongCommand {
			object_id: object_id.clone(),
			field_id: 1,
			value: 100,
		}),
	);
	env.cycle();

	// проверяем что команд по объекту не пришло
	let commands = env.get_input_commands(user2_id);
	assert!(commands.is_empty());

	// проверяем что дошла команда подтверждения создания объекта
	env.send_to_server(
		user1_id,
		C2SCommand::Created(CreatedGameObjectCommand {
			object_id: object_id.clone(),
		}),
	);
	env.cycle();

	let mut commands = env.get_input_commands(user2_id);
	assert!(matches!(commands.remove(0), S2CCommand::Create(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::Created(c) if c.object_id == object_id));

	// проверяем загрузку уже созданного объекта
	env.connect(user3_id, user3_key);
	env.send_to_server(user3_id, C2SCommand::AttachToRoom);
	env.cycle();

	let mut commands = env.get_input_commands(user3_id);
	assert!(matches!(commands.remove(0), S2CCommand::Create(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::SetLong(c) if c.object_id == object_id));
	assert!(matches!(commands.remove(0), S2CCommand::Created(c) if c.object_id == object_id));
}
