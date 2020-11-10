use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use indexmap::map::{IndexMap, MutableKeys};

use cheetah_relay_common::commands::command::{S2CCommandUnion, S2CCommandWithMeta};
use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription, ApplicationCommands};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::GameObjectId;

use crate::room::command::execute;
use crate::room::object::GameObject;
use crate::rooms::OutFrame;
use cheetah_relay_common::room::{RoomId, UserPublicKey};

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

pub trait Room {
	fn get_id(&self) -> RoomId;
	fn send_to_group(&mut self, access_groups: AccessGroups, command: S2CCommandUnion);
	fn send_to_user(&mut self, user_public_key: &UserPublicKey, command: S2CCommandUnion);
	fn on_frame_received(&mut self, user_public_key: &UserPublicKey, frame: Frame);
	fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>);
	fn return_commands(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands);
	fn register_user(&mut self, user_public_key: UserPublicKey, access_groups: AccessGroups);
	fn get_user<'a>(&'a self, user_public_key: &UserPublicKey) -> Option<&'a User>;
	fn insert_object(&mut self, object: GameObject);
	fn get_object<'a>(&'a mut self, object_id: &GameObjectId) -> Option<&'a mut GameObject>;
	fn contains_object(&self, object_id: &GameObjectId) -> bool;
	fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject>;
	fn process_objects(&self, f: &mut dyn FnMut(&GameObject) -> ());
}

impl Room for RoomImpl {
	fn get_id(&self) -> RoomId {
		self.id
	}
	
	fn send_to_group(&mut self, access_groups: AccessGroups, command: S2CCommandUnion) {
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
	
	fn send_to_user(&mut self, user_public_key: &u32, command: S2CCommandUnion) {
		match self.users.get_mut(user_public_key) {
			None => {
				log::error!("room.send_to_user - user not found {:?}", user_public_key)
			}
			Some(user) => {
				let now = Instant::now();
				if user.protocol.connected(&now) {
					let meta = self.current_meta.as_ref().unwrap();
					let channel = self.current_channel.as_ref().unwrap();
					let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
						meta: S2CMetaCommandInformation::new(user_public_key.clone(), meta),
						command,
					});
					user.protocol.out_commands_collector.add_command(channel.clone(), application_command);
				}
			}
		}
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
	
	fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
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
	
	fn process_objects(&self, mut f: &mut dyn FnMut(&GameObject) -> ()) {
		self.objects.values().for_each(|o| f(o));
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

#[cfg(test)]
mod tests {
	use std::collections::{HashMap, VecDeque};
	use std::slice::Iter;
	
	use cheetah_relay_common::commands::command::S2CCommandUnion;
	use cheetah_relay_common::protocol::frame::applications::ApplicationCommands;
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ClientOwner;
	
	use crate::room::{Room, RoomImpl, User};
	use crate::room::object::GameObject;
	use crate::rooms::OutFrame;
	use cheetah_relay_common::room::UserPublicKey;
	
	pub struct RoomStub {
		object_id_generator: u32,
		user_public_key_generator: u32,
		real_impl: RoomImpl,
		pub out_commands: VecDeque<(AccessGroups, S2CCommandUnion)>,
		pub out_commands_by_users: HashMap<UserPublicKey, VecDeque<S2CCommandUnion>>,
	}
	
	impl RoomStub {}
	
	
	impl RoomStub {
		pub fn new() -> Self {
			Self {
				object_id_generator: 0,
				user_public_key_generator: 0,
				real_impl: RoomImpl::new(0),
				out_commands: Default::default(),
				out_commands_by_users: Default::default(),
			}
		}
		
		pub fn create_user(&mut self, access_groups: AccessGroups) -> UserPublicKey {
			self.user_public_key_generator += 1;
			self.real_impl.register_user(self.user_public_key_generator, access_groups);
			self.user_public_key_generator
		}
		
		
		pub fn create_object(&mut self, owner: &UserPublicKey) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, ClientOwner::Client(owner.clone()));
			let object = GameObject {
				id: id.clone(),
				template: 0,
				access_groups: Default::default(),
				fields: Default::default(),
			};
			self.real_impl.insert_object(object);
			self.real_impl.get_object(&id).unwrap()
		}
		
		pub fn create_object_with_access_groups(&mut self, access_groups: AccessGroups) -> &mut GameObject {
			let object = self.create_object(&0);
			object.access_groups = access_groups;
			object
		}
	}
	
	
	impl Room for RoomStub {
		fn get_id(&self) -> u64 {
			unimplemented!()
		}
		
		fn send_to_group(&mut self, access_groups: AccessGroups, command: S2CCommandUnion) {
			self.out_commands.push_front((access_groups, command));
		}
		
		fn send_to_user(&mut self, user_public_key: &u32, command: S2CCommandUnion) {
			let commands = self.out_commands_by_users.entry(user_public_key.clone()).or_insert(VecDeque::new());
			commands.push_front(command);
		}
		
		fn on_frame_received(&mut self, user_public_key: &u32, frame: Frame) {
			unimplemented!()
		}
		
		fn collect_out_frames(&mut self, out_frames: &mut VecDeque<OutFrame>) {
			unimplemented!()
		}
		
		fn return_commands(&mut self, user_public_key: &u32, commands: ApplicationCommands) {
			unimplemented!()
		}
		
		fn register_user(&mut self, user_public_key: u32, access_groups: AccessGroups) {
			unimplemented!()
		}
		
		fn get_user<'a>(&'a self, user_public_key: &u32) -> Option<&'a User> {
			self.real_impl.get_user(user_public_key)
		}
		
		fn insert_object(&mut self, object: GameObject) {
			self.real_impl.insert_object(object);
		}
		
		fn get_object<'a>(&'a mut self, object_id: &GameObjectId) -> Option<&'a mut GameObject> {
			self.real_impl.get_object(object_id)
		}
		
		fn contains_object(&self, object_id: &GameObjectId) -> bool {
			self.real_impl.contains_object(object_id)
		}
		
		fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
			self.real_impl.delete_object(object_id)
		}
		
		fn process_objects(&self, mut f: &mut dyn FnMut(&GameObject)) {
			self.real_impl.process_objects(f);
		}
	}
}