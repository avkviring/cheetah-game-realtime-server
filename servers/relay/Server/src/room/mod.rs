use core::fmt;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Instant;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;
use serde::export::Formatter;

use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::commands::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_relay_common::commands::command::S2CCommand;
use cheetah_relay_common::commands::command::S2CCommandWithMeta;
use cheetah_relay_common::constants::FieldIdType;
use cheetah_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_relay_common::protocol::frame::Frame;
use cheetah_relay_common::protocol::relay::RelayProtocol;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::object::GameObjectId;
use cheetah_relay_common::room::owner::ObjectOwner;
use cheetah_relay_common::room::UserPublicKey;

use crate::room::command::execute;
use crate::room::command::long::reset_all_compare_and_set;
use crate::room::debug::tracer::CommandTracer;
use crate::room::object::GameObject;
use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::rooms::OutFrame;

pub mod command;
pub mod debug;
pub mod object;
pub mod template;
pub mod types;

pub type RoomId = u64;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	pub users: HashMap<UserPublicKey, User, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ApplicationCommandChannelType>,
	current_meta: Option<C2SMetaCommandInformation>,
	current_user: Option<UserPublicKey>,
	pub user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>,
	pub auto_create_user: bool,
	pub tracer: Rc<CommandTracer>,
	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	pub out_commands: VecDeque<(AccessGroups, S2CCommand)>,
	#[cfg(test)]
	pub out_commands_by_users: HashMap<UserPublicKey, VecDeque<S2CCommand>>,
}

pub trait RoomUserListener {
	fn register_user(&mut self, room_id: RoomId, template: &UserTemplate);
	fn connected_user(&mut self, room_id: RoomId, template: &UserTemplate);
	fn disconnected_user(&mut self, room_id: RoomId, template: &UserTemplate);
}
impl Debug for dyn RoomUserListener {
	fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
		Result::Ok(())
	}
}

#[derive(Debug)]
pub struct User {
	protocol: Option<RelayProtocol>,
	pub attached: bool,
	pub template: UserTemplate,
	pub compare_and_sets_cleaners: HashMap<(GameObjectId, FieldIdType), i64, FnvBuildHasher>,
}

#[derive(Debug)]
pub enum RoomRegisterUserError {
	AlreadyRegistered,
}

impl User {
	pub fn attach_to_room(&mut self) {
		self.attached = true;
	}
}

impl Room {
	pub fn new(template: RoomTemplate, tracer: Rc<CommandTracer>, user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> Self {
		let mut room = Room {
			id: template.id,
			auto_create_user: template.auto_create_user,
			users: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_meta: Default::default(),
			current_user: Default::default(),
			user_listeners,
			tracer,
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			#[cfg(test)]
			out_commands_by_users: Default::default(),
		};

		template.objects.into_iter().for_each(|object| {
			let game_object: GameObject = object.to_root_game_object();
			room.insert_object(game_object);
		});

		template.users.into_iter().for_each(|config| room.register_user(config).unwrap());
		room
	}

	pub fn send_to_group(&mut self, access_groups: AccessGroups, command: S2CCommand) {
		#[cfg(test)]
		self.out_commands.push_front((access_groups, command.clone()));

		let current_user_public_key = self.current_user.as_ref().unwrap_or(&u32::max_value());
		let meta = self.current_meta.as_ref().unwrap_or(&C2SMetaCommandInformation { timestamp: 0 });
		let channel_type = self
			.current_channel
			.as_ref()
			.unwrap_or(&ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let application_command = ApplicationCommand::S2CCommandWithMeta(S2CCommandWithMeta {
			meta: S2CMetaCommandInformation::new(current_user_public_key.clone(), meta),
			command: command.clone(),
		});

		let room_id = self.id;
		let tracer = self.tracer.clone();
		self.users
			.values_mut()
			.filter(|user| user.template.public_key != *current_user_public_key)
			.filter(|user| user.attached)
			.filter(|user| user.protocol.is_some())
			.filter(|user| user.template.access_groups.contains_any(&access_groups))
			.for_each(|user| {
				let protocol = user.protocol.as_mut().unwrap();
				tracer.on_s2c_command(room_id, user.template.public_key.clone(), &command);
				protocol
					.out_commands_collector
					.add_command(channel_type.clone(), application_command.clone())
			});
	}

	pub fn send_to_user(&mut self, user_public_key: &u32, commands: Vec<S2CCommand>) {
		#[cfg(test)]
		{
			let user_commands = self.out_commands_by_users.entry(user_public_key.clone()).or_insert(VecDeque::new());
			for command in &commands {
				user_commands.push_front(command.clone());
			}
		}

		match self.users.get_mut(user_public_key) {
			None => {
				log::error!("[room] send to unknown user {:?}", user_public_key)
			}
			Some(user) => {
				if let Some(ref mut protocol) = user.protocol {
					if user.attached {
						for command in commands {
							self.tracer.on_s2c_command(self.id, user.template.public_key, &command);
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
	}

	pub fn collect_out_frame(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		for (user_public_key, user) in self.users.iter_mut() {
			if let Some(ref mut protocol) = user.protocol {
				while let Some(frame) = protocol.build_next_frame(&now) {
					out_frames.push_front(OutFrame {
						user_public_key: user_public_key.clone(),
						frame,
					});
				}
			}
		}
	}

	pub fn process_in_frame(&mut self, user_public_key: &UserPublicKey, frame: Frame, now: &Instant) {
		let user = self.users.get_mut(&user_public_key);
		let mut commands = Vec::new();
		match user {
			None => {
				log::error!("[room({:?})] user({:?}) not found for input frame", self.id, user_public_key);
			}
			Some(user) => {
				self.current_user.replace(user_public_key.clone());

				let mut new_user = false;
				let protocol = &mut user.protocol;
				if protocol.is_none() {
					protocol.replace(RelayProtocol::new(now));
					new_user = true;
				}

				let protocol = protocol.as_mut().unwrap();
				protocol.on_frame_received(frame, now);
				while let Some(application_command) = protocol.in_commands_collector.get_commands().pop_back() {
					commands.push(application_command);
				}

				if new_user {
					self.current_channel.replace(ApplicationCommandChannelType::ReliableSequenceByGroup(0));
					self.current_meta.replace(C2SMetaCommandInformation { timestamp: 0 });
					let template = user.template.clone();
					self.user_connected(template);
				}
			}
		}

		for application_command in commands.into_iter() {
			match application_command.command {
				ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
					self.current_channel.replace(From::from(&application_command.channel));
					self.current_meta.replace(command_with_meta.meta.clone());
					self.tracer.on_c2s_command(self.id, user_public_key.clone(), &command_with_meta.command);
					execute(command_with_meta.command, self, &user_public_key);
				}
				_ => {
					log::error!("[room({:?})] receive unsupported command {:?}", self.id, application_command)
				}
			}
		}

		self.current_user = None;
		self.current_channel = None;
		self.current_meta = None;
	}

	pub fn register_user(&mut self, template: UserTemplate) -> Result<(), RoomRegisterUserError> {
		if self.users.contains_key(&template.public_key) {
			return Result::Err(RoomRegisterUserError::AlreadyRegistered);
		}

		self.user_listeners.iter().cloned().for_each(|listener| {
			let mut listener = (*listener).borrow_mut();
			listener.register_user(self.id.clone(), &template);
		});

		let user = User {
			protocol: None,
			attached: false,
			template,
			compare_and_sets_cleaners: Default::default(),
		};
		self.users.insert(user.template.public_key, user);
		Result::Ok(())
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
		log::info!("[room({:?})] disconnect user({:?})", self.id, user_public_key);
		self.current_user.replace(user_public_key.clone());
		match self.users.remove(user_public_key) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let ObjectOwner::User(owner) = o.id.owner {
						if owner == user.template.public_key {
							objects.push(o.id.clone());
						}
					}
				});

				for id in objects {
					self.delete_object(&id);
				}

				self.user_listeners.iter().cloned().for_each(|listener| {
					let mut listener = (*listener).borrow_mut();
					listener.disconnected_user(self.id.clone(), &user.template);
				});

				reset_all_compare_and_set(self, user.template.public_key.clone(), user.compare_and_sets_cleaners);

				if self.auto_create_user {
					self.register_user(user.template.clone()).unwrap();
				}
			}
		};
	}

	pub fn insert_object(&mut self, object: GameObject) {
		self.send_object_to_group(&object);
		self.objects.insert(object.id.clone(), object);
	}

	pub fn get_object_mut(&mut self, object_id: &GameObjectId) -> Option<&mut GameObject> {
		match self.objects.get_mut(object_id) {
			Some(object) => Option::Some(object),
			None => {
				log::error!("[room({:?})] get_object_mut - object({:?}) not found", self.id, object_id);
				Option::None
			}
		}
	}

	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}

	pub fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
		match self.objects.shift_remove(object_id) {
			None => {
				log::error!("[room({:?})] delete_object - object({:?}) not found", self.id, object_id);
				Option::None
			}
			Some(object) => {
				self.send_to_group(
					object.access_groups,
					S2CCommand::Delete(DeleteGameObjectCommand {
						object_id: object.id.clone(),
					}),
				);
				Option::Some(object)
			}
		}
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
					disconnected_user[disconnected_users_count] = u.template.public_key.clone();
					disconnected_users_count += 1;
				}
			}
		});

		for i in 0..disconnected_users_count {
			self.disconnect_user(&disconnected_user[i]);
		}
	}

	fn user_connected(&mut self, template: UserTemplate) {
		self.user_listeners.iter().cloned().for_each(|listener| {
			let mut listener = (*listener).borrow_mut();
			listener.connected_user(self.id.clone(), &template);
		});
		let user_public_key = template.public_key;
		template.objects.iter().for_each(|object_template| {
			let object = object_template.to_user_game_object(user_public_key);
			self.insert_object(object);
		});
	}

	pub fn send_object_to_group(&mut self, object: &GameObject) {
		let mut commands = Vec::new();
		object.collect_create_commands(&mut commands);
		commands.into_iter().for_each(|c| {
			self.send_to_group(object.access_groups, c);
		})
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;
	use std::time::Instant;

	use cheetah_relay_common::commands::command::event::EventCommand;
	use cheetah_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand};
	use cheetah_relay_common::protocol::frame::applications::{
		ApplicationCommand, ApplicationCommandChannel, ApplicationCommandChannelType, ApplicationCommandDescription,
	};
	use cheetah_relay_common::protocol::frame::Frame;
	use cheetah_relay_common::protocol::relay::RelayProtocol;
	use cheetah_relay_common::room::access::AccessGroups;
	use cheetah_relay_common::room::object::GameObjectId;
	use cheetah_relay_common::room::owner::ObjectOwner;
	use cheetah_relay_common::room::UserPublicKey;

	use crate::room::debug::tracer::CommandTracer;
	use crate::room::object::GameObject;
	use crate::room::template::config::{GameObjectTemplate, RoomTemplate, UserTemplate};
	use crate::room::{Room, RoomUserListener};

	impl Default for Room {
		fn default() -> Self {
			Room::new(RoomTemplate::default(), Rc::new(CommandTracer::new_with_allow_all()), Default::default())
		}
	}

	impl Room {
		pub fn new_with_template(template: RoomTemplate) -> Self {
			Room::new(template, Rc::new(CommandTracer::new_with_allow_all()), Default::default())
		}

		pub fn create_object(&mut self, owner: &UserPublicKey) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, ObjectOwner::User(owner.clone()));
			let object = GameObject::new(id.clone());
			self.insert_object(object);
			self.get_object_mut(&id).unwrap()
		}

		pub fn create_object_with_access_groups(&mut self, access_groups: AccessGroups) -> &mut GameObject {
			let object = self.create_object(&0);
			object.access_groups = access_groups;
			object
		}
	}

	#[test]
	fn should_remove_objects_when_disconnect() {
		let mut template = RoomTemplate::default();
		let user_a = template.create_user(1, AccessGroups(0b111));
		let user_b = template.create_user(2, AccessGroups(0b111));

		let mut room = Room::new_with_template(template);
		let object_a_1 = room.create_object(&user_a).id.clone();
		let object_a_2 = room.create_object(&user_a).id.clone();
		let object_b_1 = room.create_object(&user_b).id.clone();
		let object_b_2 = room.create_object(&user_b).id.clone();

		room.out_commands.clear();
		room.disconnect_user(&user_a);

		assert!(!room.contains_object(&object_a_1));
		assert!(!room.contains_object(&object_a_2));

		assert!(room.contains_object(&object_b_1));
		assert!(room.contains_object(&object_b_2));

		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_1));
		assert!(matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_2));
	}

	#[test]
	fn should_create_object_from_config() {
		let mut template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			access_groups: Default::default(),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		template.objects = vec![object_template.clone()];

		let room = Room::new_with_template(template);
		assert!(room.objects.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::Root)));
	}

	#[test]
	fn should_create_object_from_config_for_user() {
		let mut template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		let user_template = UserTemplate {
			public_key: 100,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: vec![object_template.clone()],
			unmapping: Default::default(),
		};
		template.users.push(user_template.clone());

		let mut room = Room::new_with_template(template);
		room.process_in_frame(&user_template.public_key, Frame::new(0), &Instant::now());
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::User(user_template.public_key))));
	}

	///
	/// Пользовательские объекты из шаблона должны загружаться на первый клиент при входе второго
	///
	#[test]
	fn should_load_user_object_when_connect_other_user() {
		let mut template = RoomTemplate::default();
		let object1_template = GameObjectTemplate {
			id: 100,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		let user1_template = UserTemplate {
			public_key: 1,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: vec![object1_template.clone()],
			unmapping: Default::default(),
		};

		let object2_template = GameObjectTemplate {
			id: 200,
			template: 5,
			access_groups: AccessGroups(55),
			fields: Default::default(),
			unmapping: Default::default(),
		};
		let user2_template = UserTemplate {
			public_key: 2,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: vec![object2_template.clone()],
			unmapping: Default::default(),
		};

		template.users.push(user1_template.clone());
		template.users.push(user2_template.clone());

		let mut room = Room::new_with_template(template);

		room.process_in_frame(&user1_template.public_key, Frame::new(0), &Instant::now());

		let mut frame_with_attach_to_room = Frame::new(1);
		frame_with_attach_to_room.commands.reliable.push_back(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
				meta: C2SMetaCommandInformation { timestamp: 0 },
				command: C2SCommand::AttachToRoom,
			}),
		});
		room.process_in_frame(&user1_template.public_key, frame_with_attach_to_room, &Instant::now());

		let user1 = room.get_user_mut(&user1_template.public_key).unwrap();
		let protocol = user1.protocol.as_mut().unwrap();
		assert_eq!(
			protocol
				.out_commands_collector
				.commands
				.reliable
				.pop_front()
				.unwrap()
				.command
				.get_object_id()
				.unwrap(),
			&GameObjectId::new(object1_template.id, ObjectOwner::User(user1_template.public_key))
		);
		protocol.out_commands_collector.commands.reliable.clear();
		room.process_in_frame(&user2_template.public_key, Frame::new(0), &Instant::now());
		let user1 = room.get_user_mut(&user1_template.public_key).unwrap();
		let protocol = user1.protocol.as_mut().unwrap();
		assert_eq!(
			protocol
				.out_commands_collector
				.commands
				.reliable
				.pop_front()
				.unwrap()
				.command
				.get_object_id()
				.unwrap(),
			&GameObjectId::new(object2_template.id, ObjectOwner::User(user2_template.public_key))
		);
	}

	///
	/// Регистрация пользователя после разрыва соединения если выставлен флаг автосоздания
	///
	#[test]
	fn should_register_user_after_disconnect_when_auto_create() {
		let (mut template, user_template) = create_template();
		template.auto_create_user = true;
		let mut room = Room::new_with_template(template);
		room.disconnect_user(&user_template.public_key);
		assert!(room.users.contains_key(&user_template.public_key));
	}

	pub fn from_vec(vec: Vec<u8>) -> heapless::Vec<u8, heapless::consts::U256> {
		let mut result = heapless::Vec::new();
		result.extend_from_slice(vec.as_slice()).unwrap();
		result
	}

	#[test]
	fn should_invoke_user_listeners() {
		struct TestUserListener {
			trace: String,
		}

		impl RoomUserListener for TestUserListener {
			fn register_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}r{:?}", self.trace, template.public_key);
			}

			fn connected_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}c{:?}", self.trace, template.public_key);
			}

			fn disconnected_user(&mut self, _: u64, template: &UserTemplate) {
				self.trace = format!("{}d{:?}", self.trace, template.public_key);
			}
		}

		let (template, user_template) = create_template();

		let test_listener = Rc::new(RefCell::new(TestUserListener { trace: "".to_string() }));
		let mut room = Room::new(template, Rc::new(CommandTracer::new_with_allow_all()), vec![test_listener.clone()]);
		room.process_in_frame(&user_template.public_key, Frame::new(0), &Instant::now());
		room.disconnect_user(&user_template.public_key);

		assert_eq!(test_listener.clone().borrow().trace, "r100c100d100".to_string());
	}

	#[test]
	fn should_dont_send_command_to_current_user() {
		let (template, user_template) = create_template();
		let mut room = Room::new_with_template(template);
		room.current_user.replace(user_template.public_key);
		room.current_meta.replace(C2SMetaCommandInformation { timestamp: 0 });
		room.current_channel.replace(ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let user = room.get_user_mut(&user_template.public_key).unwrap();
		user.attached = true;
		user.protocol.replace(RelayProtocol::new(&Instant::now()));

		room.send_to_group(
			user_template.access_groups.clone(),
			S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			}),
		);

		let user = room.get_user(&user_template.public_key).unwrap();
		let protocol = user.protocol.as_ref().unwrap();
		assert!(protocol.out_commands_collector.commands.reliable.is_empty());
	}

	#[test]
	fn should_send_command_to_other_user() {
		let (template, user_template) = create_template();
		let mut room = Room::new_with_template(template);
		room.current_user.replace(user_template.public_key + 1); // команда пришла от другого пользователя
		room.current_meta.replace(C2SMetaCommandInformation { timestamp: 0 });
		room.current_channel.replace(ApplicationCommandChannelType::ReliableSequenceByGroup(0));

		let user = room.get_user_mut(&user_template.public_key).unwrap();
		user.attached = true;
		user.protocol.replace(RelayProtocol::new(&Instant::now()));

		room.send_to_group(
			user_template.access_groups.clone(),
			S2CCommand::Event(EventCommand {
				object_id: Default::default(),
				field_id: 0,
				event: Default::default(),
			}),
		);

		let user = room.get_user(&user_template.public_key).unwrap();
		let protocol = user.protocol.as_ref().unwrap();
		assert_eq!(protocol.out_commands_collector.commands.reliable.len(), 1);
	}

	#[test]
	pub fn should_keep_order_object() {
		let (template, _) = create_template();
		let mut room = Room::new_with_template(template);
		room.insert_object(GameObject {
			id: GameObjectId::new(100, ObjectOwner::Root),
			template: 0,
			access_groups: Default::default(),
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			compare_and_set_owners: Default::default(),
			structures: Default::default(),
		});

		room.insert_object(GameObject {
			id: GameObjectId::new(5, ObjectOwner::Root),
			template: 0,
			access_groups: Default::default(),
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			compare_and_set_owners: Default::default(),
			structures: Default::default(),
		});

		room.insert_object(GameObject::new(GameObjectId::new(200, ObjectOwner::Root)));

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{}{}", order, o.id.id);
		});
		assert_eq!(order, "1005200");

		room.delete_object(&GameObjectId::new(100, ObjectOwner::Root));

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{}{}", order, o.id.id);
		});
		assert_eq!(order, "5200");
	}

	fn create_template() -> (RoomTemplate, UserTemplate) {
		let mut template = RoomTemplate::default();
		let user_template = UserTemplate {
			public_key: 100,
			private_key: Default::default(),
			access_groups: AccessGroups(55),
			objects: Default::default(),
			unmapping: Default::default(),
		};
		template.users.push(user_template.clone());
		(template, user_template)
	}
}
