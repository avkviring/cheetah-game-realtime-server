use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use indexmap::map::{IndexMap, MutableKeys};
#[cfg(test)]
use mockall::{automock, predicate::*};

use cheetah_relay_common::commands::command::{S2CCommandUnion, S2CCommandWithMeta};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::hash::{RoomId, UserPublicKey};
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription, ApplicationCommands};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;

use crate::room::command::execute;
use crate::room::object::GameObject;
use crate::rooms::OutFrame;

pub mod command;
pub mod object;

#[derive(Debug)]
pub struct RoomImpl {
	pub id: RoomId,
	users: HashMap<UserPublicKey, User>,
	objects: IndexMap<GameObjectId, GameObject>,
	current_channel: Option<ApplicationCommandChannel>,
	current_meta: Option<C2SMetaCommandInformation>,
	current_user: Option<UserPublicKey>,
}

#[derive(Debug)]
pub struct User {
	pub public_key: UserPublicKey,
	pub access_groups: AccessGroups,
	protocol: RelayProtocol,
}

#[cfg_attr(test, automock)]
pub trait Room {
	fn get_id(&self) -> RoomId;
	fn send(&mut self, access_groups: AccessGroups, command: S2CCommandUnion);
	fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame);
	fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>);
	fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands);
	
	fn register_user(&mut self, user_public_key: UserPublicKey, access_groups: AccessGroups);
	fn get_user<'a>(&'a self, user_public_key: &UserPublicKey) -> Option<&'a User>;
	
	fn insert_object(&mut self, object: GameObject);
	fn get_object<'a>(&'a mut self, object_id: &GameObjectId) -> Option<&'a mut GameObject>;
	fn contains_object(&self, object_id: &GameObjectId) -> bool;
	fn remove_object(&mut self, object_id: &GameObjectId) -> Option<GameObject>;
}

impl Room for RoomImpl {
	fn get_id(&self) -> RoomId {
		self.id
	}
	
	fn send(&mut self, access_groups: AccessGroups, command: S2CCommandUnion) {
		let current_user_public_key = self.current_user.as_ref().unwrap();
		let meta = self.current_meta.as_ref().unwrap();
		let channel = self.current_channel.as_ref().unwrap();
		let now = Instant::now();
		let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
			meta: S2CMetaCommandInformation::new(current_user_public_key.clone(), meta),
			command,
		});
		self.users.values_mut()
			.filter(|user| user.public_key != *current_user_public_key)
			.filter(|user| user.protocol.connected(&now))
			.filter(|user| user.access_groups.contains_any(&access_groups))
			.for_each(|user| {
				user.protocol.out_commands_collector.add_command(channel.clone(), application_command.clone())
			});
	}
	
	fn get_object(&mut self, object_id: &GameObjectId) -> Option<&mut GameObject> {
		match self.objects.get_mut(object_id) {
			Some(object) => { Option::Some(object) }
			None => {
				log::error!("game object not found {:?}", object_id);
				Option::None
			}
		}
	}
	
	fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame) {
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
					self.current_channel.replace(application_command.channel.clone());
					self.current_meta.replace(command_with_meta.meta.clone());
					self.current_user.replace(user_public_key.clone());
					execute(command_with_meta.command, self, &user_public_key);
				}
				_ => {
					log::error!("receive unsupported command from client {:?}", application_command)
				}
			}
		}
	}
	
	fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>) {
		let now = Instant::now();
		for (user_public_key, user) in self.users.iter_mut() {
			if let Some(frame) = user.protocol.build_next_frame(&now) {
				out_frames.push_front(OutFrame { user_public_key: user_public_key.clone(), frame });
			}
		}
	}
	
	fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands) {
		match self.users.get_mut(user_public_key) {
			None => {}
			Some(user) => {
				user.protocol.out_commands_collector.add_unsent_commands(commands);
			}
		}
	}
	
	fn remove_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
		self.objects.remove(object_id)
	}
	
	fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}
	
	fn insert_object(&mut self, object: GameObject) {
		self.objects.insert(object.id.clone(), object);
	}
	
	fn register_user(&mut self, user_public_key: UserPublicKey, access_groups: AccessGroups) {
		let user = User {
			public_key: user_public_key,
			access_groups,
			protocol: Default::default(),
		};
		self.users.insert(user_public_key, user);
	}
	
	fn get_user<'a>(&'a self, user_public_key: &UserPublicKey) -> Option<&'a User> {
		self.users.get(user_public_key)
	}
}


impl RoomImpl {
	pub fn new(id: RoomId) -> Self {
		RoomImpl {
			id,
			users: Default::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_meta: Default::default(),
			current_user: Default::default(),
		}
	}
}