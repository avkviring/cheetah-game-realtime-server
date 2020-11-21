use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;

use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::commands::command::S2CCommandWithMeta;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType, ApplicationCommands};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::{RoomId, UserPublicKey};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;

use crate::room::command::execute;
use crate::room::object::GameObject;
use crate::rooms::OutFrame;

pub mod command;
pub mod object;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	users: HashMap<UserPublicKey, User, FnvBuildHasher>,
	objects: IndexMap<GameObjectId, GameObject>,
	current_channel: Option<ApplicationCommandChannelType>,
	current_meta: Option<C2SMetaCommandInformation>,
	current_user: Option<UserPublicKey>,
	auto_create_user: bool,
	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	user_public_key_generator: u32,
	#[cfg(test)]
	pub out_commands: VecDeque<(AccessGroups, S2CCommand)>,
	#[cfg(test)]
	pub out_commands_by_users: HashMap<UserPublicKey, VecDeque<S2CCommand>>,
}

#[derive(Debug)]
pub struct User {
	pub public_key: UserPublicKey,
	pub access_groups: AccessGroups,
	protocol: Option<RelayProtocol>,
	attached: bool,
}

impl User {
	pub fn attach_to_room(&mut self) {
		self.attached = true;
	}
}

impl Room {
	pub fn new(id: RoomId, auto_create_user: bool) -> Self {
		Room {
			id,
			users: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_meta: Default::default(),
			current_user: Default::default(),
			auto_create_user,
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			user_public_key_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			#[cfg(test)]
			out_commands_by_users: Default::default(),
		}
	}
	
	fn get_id(&self) -> RoomId {
		self.id
	}
	
	pub fn send_to_group(&mut self, access_groups: AccessGroups, command: S2CCommand) {
		#[cfg(test)]
			self.out_commands.push_front((access_groups, command.clone()));
		
		#[cfg(test)]
		if self.current_user.is_none() {
			return;
		}
		
		let current_user_public_key = self.current_user.as_ref().unwrap();
		let meta = self.current_meta.as_ref().unwrap();
		let channel_type = self.current_channel.as_ref().unwrap();
		let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
			meta: S2CMetaCommandInformation::new(current_user_public_key.clone(), meta),
			command,
		});
		self.users.values_mut()
			.filter(|user| user.public_key != *current_user_public_key)
			.filter(|user| user.attached)
			.filter(|user| user.protocol.is_some())
			.filter(|user| user.access_groups.contains_any(&access_groups))
			.for_each(|user| {
				let protocol = user.protocol.as_mut().unwrap();
				protocol.out_commands_collector.add_command(channel_type.clone(), application_command.clone())
			});
	}
	
	pub fn send_to_user(&mut self, user_public_key: &u32, command: S2CCommand) {
		#[cfg(test)]
			{
				let commands = self.out_commands_by_users.entry(user_public_key.clone()).or_insert(VecDeque::new());
				commands.push_front(command.clone());
			}
		
		match self.users.get_mut(user_public_key) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_public_key)
			}
			Some(user) => {
				if let Some(ref mut protocol) = user.protocol {
					if user.attached {
						let meta = self.current_meta.as_ref().unwrap();
						let channel = self.current_channel.as_ref().unwrap();
						let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
							meta: S2CMetaCommandInformation::new(user_public_key.clone(), meta),
							command,
						});
						protocol.out_commands_collector.add_command(channel.clone(), application_command.clone());
					}
				}
			}
		}
	}
	
	pub fn collect_out_frame(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		for (user_public_key, user) in self.users.iter_mut() {
			if let Some(ref mut protocol) = user.protocol {
				if let Some(frame) = protocol.build_next_frame(&now) {
					out_frames.push_front(OutFrame { user_public_key: user_public_key.clone(), frame });
				}
			}
		}
	}
	
	pub fn process_in_frame(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		self.auto_create_user(&user_public_key);
		
		let user = self.users.get_mut(&user_public_key);
		
		let mut commands = VecDeque::new();
		match user {
			None => {
				log::error!("[room ({:?})] user({:?}) not found for input frame", self.id, user_public_key);
			}
			Some(user) => {
				let protocol = &mut user.protocol;
				if protocol.is_none() {
					protocol.replace(RelayProtocol::new(now));
				}
				let protocol = protocol.as_mut().unwrap();
				protocol.on_frame_received(frame, now);
				while let Some(application_command) = protocol.in_commands_collector.get_commands().pop_back() {
					commands.push_front(application_command);
				}
			}
		}
		
		for application_command in commands {
			match application_command.command {
				ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
					self.current_channel.replace(From::from(&application_command.channel));
					self.current_meta.replace(command_with_meta.meta.clone());
					self.current_user.replace(user_public_key.clone());
					execute(command_with_meta.command, self, &user_public_key);
				}
				_ => {
					log::error!("[room ({:?})] receive unsupported command {:?}", self.id, application_command)
				}
			}
		}
	}
	
	fn auto_create_user(&mut self, user_public_key: &UserPublicKey) {
		if !self.auto_create_user {
			return;
		}
		if !self.users.contains_key(&user_public_key) {
			self.register_user(user_public_key.clone(), AccessGroups(0b111111));
		}
	}
	
	pub fn send_to_user_first(&mut self, user_public_key: &UserPublicKey, commands: ApplicationCommands) {
		match self.users.get_mut(user_public_key) {
			None => {}
			Some(user) => {
				match user.protocol {
					None => {}
					Some(ref mut protocol) => {
						protocol.out_commands_collector.add_unsent_commands(commands);
					}
				}
			}
		}
	}
	
	pub fn register_user(&mut self, user_public_key: UserPublicKey, access_groups: AccessGroups) {
		let user = User {
			public_key: user_public_key,
			access_groups,
			protocol: None,
			attached: false,
		};
		self.users.insert(user_public_key, user);
	}
	
	pub fn get_user(&self, user_public_key: &UserPublicKey) -> Option<&User> {
		self.users.get(user_public_key)
	}
	
	pub fn get_user_mut(&mut self, user_public_key: &UserPublicKey) -> Option<&mut User> {
		self.users.get_mut(user_public_key)
	}
	
	
	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_user(&mut self, user_public_key: &UserPublicKey) {
		log::info!("[room ({:?})] disconnect user({:?})", self.id, user_public_key);
		match self.users.remove(user_public_key) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let ObjectOwner::User(owner) = o.id.owner {
						if owner == user.public_key {
							objects.push((o.id.clone(), o.access_groups.clone()));
						}
					}
				});
				
				for (id, access_groups) in objects {
					self.delete_object(&id);
					self.send_to_group(access_groups, S2CCommand::Delete(DeleteGameObjectCommand { object_id: id }));
				}
			}
		};
	}
	
	pub fn insert_object(&mut self, object: GameObject) {
		self.objects.insert(object.id.clone(), object);
	}
	
	pub fn get_object(&mut self, object_id: &GameObjectId) -> Option<&mut GameObject> {
		match self.objects.get_mut(object_id) {
			Some(object) => { Option::Some(object) }
			None => {
				log::error!("[room] game object not found {:?}", object_id);
				Option::None
			}
		}
	}
	
	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}
	
	pub fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
		self.objects.remove(object_id)
	}
	
	pub fn process_objects(&self, f: &mut dyn FnMut(&GameObject) -> ()) {
		self.objects.iter().for_each(|(_, o)| f(o));
	}
	
	///
	/// Тактируем протоколы пользователей и определяем дисконнекты
	///
	pub fn cycle(&mut self, now: &Instant) {
		let mut disconnected_user: [u32; 10] = [0; 10];
		let mut disconnected_users_count = 0;
		self.users.values_mut().for_each(|u| {
			if let Some(ref mut protocol) = u.protocol {
				if protocol.disconnected(now) && disconnected_users_count < disconnected_user.len() {
					disconnected_user[disconnected_users_count] = u.public_key.clone();
					disconnected_users_count += 1;
				}
			}
		});
		
		for i in 0..disconnected_users_count {
			self.disconnect_user(&disconnected_user[i]);
		}
	}
}


#[cfg(test)]
mod tests {
	use cheetah_relay_common::commands::command::S2CCommand;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;
	
	use crate::room::object::GameObject;
	use crate::room::Room;
	
	impl Room {
		pub fn create_user(&mut self, access_groups: AccessGroups) -> UserPublicKey {
			self.user_public_key_generator += 1;
			self.register_user(self.user_public_key_generator, access_groups);
			self.user_public_key_generator
		}
		
		pub fn create_object(&mut self, owner: &UserPublicKey) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, ObjectOwner::User(owner.clone()));
			let object = GameObject {
				id: id.clone(),
				template: 0,
				access_groups: Default::default(),
				fields: Default::default(),
			};
			self.insert_object(object);
			self.get_object(&id).unwrap()
		}
		
		pub fn create_object_with_access_groups(&mut self, access_groups: AccessGroups) -> &mut GameObject {
			let object = self.create_object(&0);
			object.access_groups = access_groups;
			object
		}
	}
	
	
	#[test]
	fn should_remove_objects_when_disconnect() {
		let mut room = Room::new(0);
		
		let user_a = room.create_user(AccessGroups(0b111));
		let object_a_1 = room.create_object(&user_a).id.clone();
		let object_a_2 = room.create_object(&user_a).id.clone();
		
		let user_b = room.create_user(AccessGroups(0b111));
		let object_b_1 = room.create_object(&user_b).id.clone();
		let object_b_2 = room.create_object(&user_b).id.clone();
		
		room.out_commands.clear();
		room.disconnect_user(&user_a);
		
		assert!(!room.contains_object(&object_a_1));
		assert!(!room.contains_object(&object_a_2));
		
		
		assert!(room.contains_object(&object_b_1));
		assert!(room.contains_object(&object_b_2));
		println!("{:?}", room.out_commands);
		
		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_1));
		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_2));
	}
}