use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use indexmap::map::{IndexMap, MutableKeys};

use cheetah_relay_common::commands::command::{S2CCommandUnion, S2CCommandWithMeta};
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::hash::{RoomId, UserPublicKey};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommands};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;

use crate::room::command::{CommandContext, execute};
use crate::room::object::GameObject;
use crate::room::object::server_object_id::ServerGameObjectId;
use crate::rooms::OutFrame;

pub mod command;
pub mod object;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	users: HashMap<UserPublicKey, User>,
	objects: IndexMap<ServerGameObjectId, GameObject>,
}

#[derive(Debug)]
pub struct User {
	pub public_key: UserPublicKey,
	pub access_groups: AccessGroups,
	protocol: RelayProtocol,
}


#[derive(Debug)]
pub enum GameObjectCreateErrors {
	AlreadyExists(ServerGameObjectId)
}


impl Room {
	pub fn new(id: RoomId) -> Self {
		Room {
			id,
			users: Default::default(),
			objects: Default::default(),
		}
	}
	
	pub fn register_user(&mut self, public_key: UserPublicKey, access_groups: AccessGroups) {
		let user = User {
			public_key,
			access_groups,
			protocol: Default::default(),
		};
		self.users.insert(public_key, user);
	}
	
	pub fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>) {
		let now = Instant::now();
		for (user_public_key, user) in self.users.iter_mut() {
			if let Some(frame) = user.protocol.build_next_frame(&now) {
				out_frames.push_front(OutFrame { user_public_key: user_public_key.clone(), frame });
			}
		}
	}
	pub fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands) {
		match self.users.get_mut(user_public_key) {
			None => {}
			Some(user) => {
				user.protocol.out_commands_collector.add_unsent_commands(commands);
			}
		}
	}
	
	pub fn send_to_clients<F>(&mut self,
							  access_group: AccessGroups,
							  game_object_id: ServerGameObjectId,
							  context: &CommandContext,
							  mut command_factory: F) where F: FnMut(&UserPublicKey, ClientGameObjectId) -> S2CCommandUnion {
		let current_user_public_key = context.current_client.unwrap().public_key;
		let meta = &context.meta.as_ref().unwrap();
		let now = Instant::now();
		self.users.values_mut()
			.filter(|user| user.public_key != current_user_public_key)
			.filter(|user| user.protocol.connected(&now))
			.filter(|user| user.access_groups.contains_any(&access_group))
			.for_each(|user| {
				let command = command_factory(&user.public_key, game_object_id.to_client_object_id(Option::Some(user.public_key)));
				user.protocol.out_commands_collector.add_command(
					context.channel.clone(),
					ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
						meta: S2CMetaCommandInformation::new(current_user_public_key, meta),
						command,
					}),
				)
			});
	}
	
	
	pub fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame) {
		let user = self.users.get_mut(&user_public_key);
		let mut commands = VecDeque::new();
		match user {
			None => {
				log::error!("user not found for frame {:?}", user_public_key);
			}
			Some(user) => {
				let protocol = &mut user.protocol;
				protocol.on_frame_received(frame, &Instant::now());
				while let Some(application_command) = protocol.in_commands_collector.get_commands().pop_back() {
					commands.push_front(application_command);
				}
			}
		}
		
		for application_command in commands {
			match application_command.command {
				ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
					let context = CommandContext {
						current_client: None,
						channel: application_command.channel,
						meta: Option::Some(command_with_meta.meta),
					};
					execute(command_with_meta.command, self, &context);
				}
				_ => {
					log::error!("receive unsupported command from client {:?}", application_command)
				}
			}
		}
	}
	
	///
	/// Получение игрового объекта с проверкой прав доступа
	/// TODO - добавить проверку прав
	///
	pub fn get_object(&mut self,
					  user_public_key: UserPublicKey,
					  object_id: &ClientGameObjectId) ->
					  Option<&mut GameObject> {
		let object_id = ServerGameObjectId::new(Option::Some(user_public_key), object_id);
		match self.objects.get_full_mut2(&object_id) {
			Some((_, _, object)) => { Option::Some(object) }
			None => {
				log::error!("game object not found {:?} {:?}", user_public_key, object_id);
				Option::None
			}
		}
	}
}