use core::fmt;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::Instant;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;

use cheetah_matches_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_matches_relay_common::commands::command::unload::DeleteGameObjectCommand;
use cheetah_matches_relay_common::commands::command::S2CCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannelType};
use cheetah_matches_relay_common::protocol::frame::Frame;
use cheetah_matches_relay_common::protocol::others::user_id::UserAndRoomId;
use cheetah_matches_relay_common::protocol::relay::RelayProtocol;
#[cfg(test)]
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::ObjectOwner;
use cheetah_matches_relay_common::room::{RoomId, UserId};

use crate::room::command::execute;
use crate::room::command::long::reset_all_compare_and_set;
use crate::room::object::{GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::room::template::permission::PermissionManager;
use crate::rooms::OutFrame;

pub mod command;
pub mod debug;
pub mod object;
pub mod sender;
pub mod template;
pub mod types;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	pub permission_manager: Rc<RefCell<PermissionManager>>,
	pub users: HashMap<UserId, User, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ApplicationCommandChannelType>,
	current_meta: Option<C2SMetaCommandInformation>,
	current_user: Option<UserId>,
	pub user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>,
	pub user_id_generator: UserId,
	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	///
	/// Исходящие команды, без проверки на прав доступа, наличия пользователей и так далее
	///
	pub out_commands: VecDeque<(AccessGroups, S2CCommand)>,
}

pub trait RoomUserListener {
	fn register_user(&mut self, room_id: RoomId, user_id: UserId, template: UserTemplate);
	fn disconnected_user(&mut self, room_id: RoomId, user_id: UserId);
}
impl Debug for dyn RoomUserListener {
	fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
		Result::Ok(())
	}
}

#[derive(Debug)]
pub struct User {
	pub id: UserId,
	pub(crate) protocol: Option<RelayProtocol>,
	pub attached: bool,
	pub template: UserTemplate,
	pub compare_and_sets_cleaners: HashMap<(GameObjectId, FieldId), i64, FnvBuildHasher>,
}

impl User {
	pub fn attach_to_room(&mut self) {
		self.attached = true;
	}
	pub fn detach_from_room(&mut self) {
		self.attached = false;
	}
}

impl Room {
	pub fn new(id: RoomId, template: RoomTemplate, user_listeners: Vec<Rc<RefCell<dyn RoomUserListener>>>) -> Self {
		let mut room = Room {
			id,
			users: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_meta: Default::default(),
			current_user: Default::default(),
			user_listeners,
			permission_manager: Rc::new(RefCell::new(PermissionManager::new(&template.permissions))),
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			user_id_generator: 0,
		};

		template.objects.into_iter().for_each(|object| {
			let game_object: GameObject = object.to_root_game_object();
			room.insert_object(game_object);
		});
		room
	}

	pub fn collect_out_frame(&mut self, out_frames: &mut VecDeque<OutFrame>, now: &Instant) {
		for (user_id, user) in self.users.iter_mut() {
			if let Some(ref mut protocol) = user.protocol {
				while let Some(frame) = protocol.build_next_frame(&now) {
					out_frames.push_front(OutFrame {
						user_and_room_id: UserAndRoomId {
							user_id: *user_id,
							room_id: self.id,
						},
						frame,
					});
				}
			}
		}
	}

	pub fn process_in_frame(&mut self, user_id: UserId, frame: Frame, now: &Instant) {
		let user = self.users.get_mut(&user_id);
		let mut commands = Vec::new();
		match user {
			None => {
				log::error!("[room({:?})] user({:?}) not found for input frame", self.id, user_id);
			}
			Some(user) => {
				self.current_user.replace(user_id.clone());

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
					self.current_meta.replace(C2SMetaCommandInformation::default());
					let user_id = user.id;
					let template = user.template.clone();
					self.on_user_connect(user_id, template);
				}
			}
		}

		for application_command in commands.into_iter() {
			match application_command.command {
				ApplicationCommand::C2SCommandWithMeta(command_with_meta) => {
					self.current_channel.replace(From::from(&application_command.channel));
					self.current_meta.replace(command_with_meta.meta.clone());
					execute(command_with_meta.command, self, user_id);
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

	pub fn register_user(&mut self, template: UserTemplate) -> UserId {
		self.user_id_generator += 1;
		let user_id = self.user_id_generator;
		let user = User {
			id: user_id,
			protocol: None,
			attached: false,
			template,
			compare_and_sets_cleaners: Default::default(),
		};

		self.user_listeners.iter().cloned().for_each(|listener| {
			let mut listener = (*listener).borrow_mut();
			listener.register_user(self.id.clone(), user.id, user.template.clone());
		});

		self.users.insert(user_id, user);
		user_id
	}

	pub fn get_user(&self, user_id: UserId) -> Option<&User> {
		self.users.get(&user_id)
	}

	pub fn get_user_mut(&mut self, user_id: UserId) -> Option<&mut User> {
		self.users.get_mut(&user_id)
	}

	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_user(&mut self, user_id: UserId) {
		log::info!("[room({:?})] disconnect user({:?})", self.id, user_id);
		self.current_user.replace(user_id.clone());
		match self.users.remove(&user_id) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let ObjectOwner::User(owner) = o.id.owner {
						if owner == user.id {
							objects.push(o.id.clone());
						}
					}
				});

				for id in objects {
					self.delete_object(&id);
				}

				self.user_listeners.iter().cloned().for_each(|listener| {
					let mut listener = (*listener).borrow_mut();
					listener.disconnected_user(self.id.clone(), user.id);
				});

				reset_all_compare_and_set(self, user.id.clone(), user.compare_and_sets_cleaners);
			}
		};
	}

	pub fn insert_object(&mut self, object: GameObject) {
		self.objects.insert(object.id.clone(), object);
	}

	pub fn get_object_mut(&mut self, object_id: &GameObjectId) -> Option<&mut GameObject> {
		self.objects.get_mut(object_id)
	}

	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}

	pub fn delete_object(&mut self, object_id: &GameObjectId) -> Option<GameObject> {
		let current_user = self.current_user.clone();
		match self.objects.shift_remove(object_id) {
			None => {
				log::error!("[room({:?})] delete_object - object({:?}) not found", self.id, object_id);
				Option::None
			}
			Some(object) => {
				self.send(
					object.access_groups,
					object.template,
					[S2CommandWithFieldInfo {
						field: None,
						command: S2CCommand::Delete(DeleteGameObjectCommand {
							object_id: object.id.clone(),
						}),
					}]
					.iter(),
					|user| {
						if let Some(user_id) = current_user {
							user.id != user_id
						} else {
							true
						}
					},
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
	/// true - если room была изменена и требуется отправка команды
	///
	pub fn cycle(&mut self, now: &Instant) -> bool {
		let mut disconnected_user: [UserId; 10] = [0; 10];
		let mut disconnected_users_count = 0;
		self.users.values_mut().for_each(|u| {
			if let Some(ref mut protocol) = u.protocol {
				if protocol.disconnected(now) && disconnected_users_count < disconnected_user.len() {
					disconnected_user[disconnected_users_count] = u.id.clone();
					disconnected_users_count += 1;
				}
			}
		});

		for i in 0..disconnected_users_count {
			self.disconnect_user(disconnected_user[i]);
		}

		disconnected_users_count > 0
	}

	fn on_user_connect(&mut self, user_id: UserId, template: UserTemplate) {
		template.objects.iter().for_each(|object_template| {
			let object = object_template.create_user_game_object(user_id);
			let mut commands = Vec::new();
			object.collect_create_commands(&mut commands);
			let template = object.template;
			let access_groups = object.access_groups;
			self.send(access_groups, template, commands.iter(), |_user| true);
			self.insert_object(object);
		});
	}
}

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::collections::VecDeque;
	use std::rc::Rc;
	use std::time::Instant;

	use cheetah_matches_relay_common::commands::command::meta::c2s::C2SMetaCommandInformation;
	use cheetah_matches_relay_common::commands::command::{C2SCommand, C2SCommandWithMeta, S2CCommand, S2CCommandWithMeta};
	use cheetah_matches_relay_common::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
	use cheetah_matches_relay_common::protocol::frame::Frame;
	use cheetah_matches_relay_common::protocol::relay::RelayProtocol;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::ObjectOwner;
	use cheetah_matches_relay_common::room::UserId;

	use crate::room::object::GameObject;
	use crate::room::template::config::{GameObjectTemplate, Permission, RoomTemplate, UserTemplate};
	use crate::room::types::FieldType;
	use crate::room::{Room, RoomUserListener};

	impl Default for Room {
		fn default() -> Self {
			Room::new(0, RoomTemplate::default(), Default::default())
		}
	}

	impl Room {
		pub fn from_template(template: RoomTemplate) -> Self {
			Room::new(0, template, Default::default())
		}

		pub fn create_object(&mut self, owner: UserId, access_groups: AccessGroups) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, ObjectOwner::User(owner.clone()));
			let mut object = GameObject::new(id.clone());
			object.access_groups = access_groups;
			self.insert_object(object);
			self.get_object_mut(&id).unwrap()
		}

		pub fn mark_as_connected(&mut self, user_id: UserId) {
			match self.get_user_mut(user_id) {
				None => {}
				Some(user) => {
					user.protocol = Option::Some(RelayProtocol::new(&Instant::now()));
					user.attached = true;
				}
			}
		}

		pub fn get_user_out_commands(&self, user_id: UserId) -> VecDeque<S2CCommand> {
			self.get_user(user_id)
				.unwrap()
				.protocol
				.as_ref()
				.unwrap()
				.out_commands_collector
				.commands
				.reliable
				.iter()
				.map(|c| &c.command)
				.map(|c| match c {
					ApplicationCommand::TestSimple(_) => None,
					ApplicationCommand::TestObject(_, _) => None,
					ApplicationCommand::S2CCommandWithMeta(c) => Some(c.command.clone()),
					ApplicationCommand::C2SCommandWithMeta(_) => None,
				})
				.filter(|c| c.is_some())
				.map(|c| c.unwrap())
				.collect()
		}

		pub fn get_user_out_commands_with_meta(&self, user_id: UserId) -> VecDeque<S2CCommandWithMeta> {
			self.get_user(user_id)
				.unwrap()
				.protocol
				.as_ref()
				.unwrap()
				.out_commands_collector
				.commands
				.reliable
				.iter()
				.map(|c| &c.command)
				.map(|c| match c {
					ApplicationCommand::TestSimple(_) => None,
					ApplicationCommand::TestObject(_, _) => None,
					ApplicationCommand::S2CCommandWithMeta(c) => Some(c.clone()),
					ApplicationCommand::C2SCommandWithMeta(_) => None,
				})
				.filter(|c| c.is_some())
				.map(|c| c.unwrap())
				.collect()
		}

		pub fn clear_user_out_commands(&mut self, user_id: UserId) {
			self.get_user_mut(user_id)
				.unwrap()
				.protocol
				.as_mut()
				.unwrap()
				.out_commands_collector
				.commands
				.reliable
				.clear();
		}
	}

	#[test]
	fn should_remove_objects_when_disconnect() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b111);
		let mut room = Room::from_template(template);
		let user_a = room.register_user(UserTemplate::stub(access_groups));
		let user_b = room.register_user(UserTemplate::stub(access_groups));
		let object_a_1 = room.create_object(user_a, access_groups).id.clone();
		let object_a_2 = room.create_object(user_a, access_groups).id.clone();
		let object_b_1 = room.create_object(user_b, access_groups).id.clone();
		let object_b_2 = room.create_object(user_b, access_groups).id.clone();

		room.out_commands.clear();
		room.disconnect_user(user_a);

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
			groups: Default::default(),
			fields: Default::default(),
		};
		template.objects = vec![object_template.clone()];

		let room = Room::from_template(template);
		assert!(room.objects.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::Root)));
	}

	#[test]
	fn should_create_object_from_config_for_user() {
		let template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user_template = UserTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: vec![object_template.clone()],
		};
		let mut room = Room::from_template(template);
		let user_id = room.register_user(user_template.clone());
		room.process_in_frame(user_id, Frame::new(0), &Instant::now());
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, ObjectOwner::User(user_id))));
	}

	///
	/// Пользовательские объекты из шаблона должны загружаться на первый клиент при входе второго
	///
	#[test]
	fn should_load_user_object_when_connect_other_user() {
		let template = RoomTemplate::default();
		let object1_template = GameObjectTemplate {
			id: 100,
			template: 5,
			groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user1_template = UserTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: vec![object1_template.clone()],
		};

		let object2_template = GameObjectTemplate {
			id: 200,
			template: 5,
			groups: AccessGroups(55),
			fields: Default::default(),
		};
		let user2_template = UserTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: vec![object2_template.clone()],
		};

		let mut room = Room::from_template(template);
		let user1_id = room.register_user(user1_template.clone());
		let user2_id = room.register_user(user2_template.clone());
		room.process_in_frame(user1_id, Frame::new(0), &Instant::now());

		let mut frame_with_attach_to_room = Frame::new(1);
		frame_with_attach_to_room.commands.reliable.push_back(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::C2SCommandWithMeta(C2SCommandWithMeta {
				meta: C2SMetaCommandInformation::default(),
				command: C2SCommand::AttachToRoom,
			}),
		});
		room.process_in_frame(user1_id, frame_with_attach_to_room, &Instant::now());

		let user1 = room.get_user_mut(user1_id).unwrap();
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
			&GameObjectId::new(object1_template.id, ObjectOwner::User(user1_id))
		);
		protocol.out_commands_collector.commands.reliable.clear();
		room.process_in_frame(user2_id, Frame::new(0), &Instant::now());
		let user1 = room.get_user_mut(user1_id).unwrap();
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
			&GameObjectId::new(object2_template.id, ObjectOwner::User(user2_id))
		);
	}

	pub fn from_vec(vec: Vec<u8>) -> heapless::Vec<u8, 256> {
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
			fn register_user(&mut self, _room_id: u64, user_id: u16, _: UserTemplate) {
				self.trace = format!("{}r{}", self.trace, user_id);
			}

			fn disconnected_user(&mut self, _room_id: u64, user_id: u16) {
				self.trace = format!("{}d{}", self.trace, user_id);
			}
		}

		let (template, user_template) = create_template();

		let test_listener = Rc::new(RefCell::new(TestUserListener { trace: "".to_string() }));
		let mut room = Room::new(0, template, vec![test_listener.clone()]);
		let user_id = room.register_user(user_template.clone());
		room.process_in_frame(user_id, Frame::new(0), &Instant::now());
		room.disconnect_user(user_id);

		assert_eq!(test_listener.clone().borrow().trace, format!("r{}d{}", user_id, user_id).to_string());
	}

	#[test]
	pub fn should_keep_order_object() {
		let (template, user_template) = create_template();
		let mut room = Room::from_template(template);
		room.register_user(user_template);
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

	///
	/// При загрузки пользовательских предопределенных объектов должны быть учтены правила доступа
	///
	#[test]
	pub fn should_apply_permissions_for_self_object() {
		let mut template = RoomTemplate::default();
		let groups = AccessGroups(55);

		let mut user1_template = UserTemplate::stub(groups);
		let object1_template = user1_template.configure_object(1, 100, groups);
		let allow_field_id = 5;
		let deny_field_id = 10;
		object1_template.fields.longs.insert(allow_field_id, 555);
		object1_template.fields.longs.insert(deny_field_id, 111);
		template
			.permissions
			.set_permission(100, &deny_field_id, FieldType::Long, &groups, Permission::Deny);

		let mut room = Room::from_template(template);
		let user1_id = room.register_user(user1_template.clone());
		let user2 = UserTemplate::stub(groups);
		let user2_id = room.register_user(user2.clone());
		room.mark_as_connected(user2_id);
		room.on_user_connect(user1_id, user1_template.clone());

		let commands = room.get_user_out_commands(user2_id);

		assert!(matches!(commands.get(0), Some(S2CCommand::Create(_))));
		assert!(matches!(commands.get(1), Some(S2CCommand::SetLong(command)) if command.field_id == allow_field_id));
		assert!(matches!(commands.get(2), Some(S2CCommand::Created(_))));
	}

	pub fn create_template() -> (RoomTemplate, UserTemplate) {
		let template = RoomTemplate::default();
		let user_template = UserTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: Default::default(),
		};
		(template, user_template)
	}
}
