use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::rc::Rc;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;

use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::unload::DeleteGameObjectCommand;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::commands::output::OutCommand;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use cheetah_matches_relay_common::protocol::frame::channel::ChannelType;
#[cfg(test)]
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::debug::tracer::CommandTracerSessions;
use crate::room::command::execute;
use crate::room::command::long::reset_all_compare_and_set;
use crate::room::object::{GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::{RoomTemplate, UserTemplate};
use crate::room::template::permission::PermissionManager;

pub mod action;
pub mod command;
pub mod object;
pub mod sender;
pub mod template;
pub mod types;

#[derive(Debug)]
pub struct Room {
	pub id: RoomId,
	pub permission_manager: Rc<RefCell<PermissionManager>>,
	pub users: HashMap<RoomMemberId, User, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ChannelType>,
	current_user: Option<RoomMemberId>,
	pub user_id_generator: RoomMemberId,
	pub command_trace_session: Rc<RefCell<CommandTracerSessions>>,
	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	///
	/// Исходящие команды, без проверки на прав доступа, наличия пользователей и так далее
	///
	pub out_commands: VecDeque<(AccessGroups, S2CCommand)>,
}

#[derive(Debug)]
pub struct User {
	pub id: RoomMemberId,
	pub connected: bool,
	pub attached: bool,
	pub template: UserTemplate,
	pub compare_and_sets_cleaners: HashMap<(GameObjectId, FieldId), i64, FnvBuildHasher>,
	pub out_commands: VecDeque<OutCommand>,
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
	pub fn new(id: RoomId, template: RoomTemplate) -> Self {
		let mut room = Room {
			id,
			users: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_user: Default::default(),
			permission_manager: Rc::new(RefCell::new(PermissionManager::new(&template.permissions))),
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			user_id_generator: 0,
			command_trace_session: Default::default(),
		};

		template.objects.into_iter().for_each(|object| {
			let game_object: GameObject = object.to_root_game_object();
			room.insert_object(game_object);
		});
		room
	}

	///
	/// Получить команды для отправки в сеть
	///
	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomMemberId, &mut VecDeque<OutCommand>),
	{
		for (user_id, user) in self.users.iter_mut() {
			collector(user_id, &mut user.out_commands);
		}
	}

	///
	/// Обработать входящие команды
	///
	pub fn execute_commands(&mut self, user_id: RoomMemberId, commands: &[CommandWithChannel]) {
		let user = self.users.get_mut(&user_id);
		match user {
			None => {
				log::error!("[room({:?})] user({:?}) not found for input frame", self.id, user_id);
			}
			Some(user) => {
				self.current_user.replace(user_id);

				let connected_now = !user.connected;
				user.connected = true;

				if connected_now {
					self.current_channel
						.replace(ChannelType::ReliableSequence(ChannelGroup(0)));
					let user_id = user.id;
					let template = user.template.clone();
					self.on_user_connect(user_id, template);
				}
			}
		}

		let tracer = self.command_trace_session.clone();
		for command_with_channel in commands {
			match &command_with_channel.both_direction_command {
				BothDirectionCommand::C2S(command) => {
					self.current_channel.replace(From::from(&command_with_channel.channel));
					tracer.borrow_mut().collect_c2s(&self.objects, user_id, &command);
					execute(command, self, user_id);
				}
				_ => {
					log::error!("[room({:?})] receive unsupported command {:?}", self.id, command_with_channel)
				}
			}
		}

		self.current_user = None;
		self.current_channel = None;
	}

	pub fn register_user(&mut self, template: UserTemplate) -> RoomMemberId {
		self.user_id_generator += 1;
		let user_id = self.user_id_generator;
		let user = User {
			id: user_id,
			connected: false,
			attached: false,
			template,
			compare_and_sets_cleaners: Default::default(),
			out_commands: Default::default(),
		};
		self.users.insert(user_id, user);
		user_id
	}

	pub fn get_user(&self, user_id: RoomMemberId) -> Option<&User> {
		self.users.get(&user_id)
	}

	pub fn get_user_mut(&mut self, user_id: RoomMemberId) -> Option<&mut User> {
		self.users.get_mut(&user_id)
	}

	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_user(&mut self, user_id: RoomMemberId) {
		log::info!("[room({:?})] disconnect user({:?})", self.id, user_id);
		self.current_user.replace(user_id);
		match self.users.remove(&user_id) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let GameObjectOwner::User(owner) = o.id.owner {
						if owner == user.id {
							objects.push(o.id.clone());
						}
					}
				});

				for id in objects {
					self.delete_object(&id);
				}

				reset_all_compare_and_set(self, user.id, user.compare_and_sets_cleaners);
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
		let current_user = self.current_user;
		match self.objects.shift_remove(object_id) {
			None => {
				log::error!("[room({:?})] delete_object - object({:?}) not found", self.id, object_id);
				Option::None
			}
			Some(object) => {
				self.send_to_users(
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

	pub fn process_objects(&self, f: &mut dyn FnMut(&GameObject)) {
		self.objects.iter().for_each(|(_, o)| f(o));
	}

	fn on_user_connect(&mut self, user_id: RoomMemberId, template: UserTemplate) {
		template.objects.iter().for_each(|object_template| {
			let object = object_template.create_user_game_object(user_id);
			let mut commands = Vec::new();
			object.collect_create_commands(&mut commands);
			let template = object.template;
			let access_groups = object.access_groups;
			self.send_to_users(access_groups, template, commands.iter(), |_user| true);
			self.insert_object(object);
		});
	}
}

#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use cheetah_matches_relay_common::commands::c2s::C2SCommand;
	use cheetah_matches_relay_common::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use cheetah_matches_relay_common::protocol::frame::channel::Channel;
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::object::GameObject;
	use crate::room::template::config::{GameObjectTemplate, Permission, RoomTemplate, UserTemplate};
	use crate::room::Room;

	impl Default for Room {
		fn default() -> Self {
			Room::new(0, RoomTemplate::default())
		}
	}

	impl Room {
		pub fn from_template(template: RoomTemplate) -> Self {
			Room::new(0, template)
		}

		pub fn create_object(&mut self, owner: RoomMemberId, access_groups: AccessGroups) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, GameObjectOwner::User(owner));
			let mut object = GameObject::new(id.clone());
			object.access_groups = access_groups;
			self.insert_object(object);
			self.get_object_mut(&id).unwrap()
		}

		pub fn mark_as_connected(&mut self, user_id: RoomMemberId) {
			match self.get_user_mut(user_id) {
				None => {}
				Some(user) => {
					user.connected = true;
					user.attached = true;
				}
			}
		}

		pub fn get_user_out_commands(&self, user_id: RoomMemberId) -> VecDeque<S2CCommand> {
			self.get_user(user_id)
				.unwrap()
				.out_commands
				.iter()
				.map(|c| &c.command)
				.map(|c| match c {
					BothDirectionCommand::S2CWithCreator(c) => Some(c.command.clone()),
					BothDirectionCommand::C2S(_) => None,
				})
				.flatten()
				.collect()
		}

		pub fn get_user_out_commands_with_meta(&self, user_id: RoomMemberId) -> VecDeque<S2CCommandWithCreator> {
			self.get_user(user_id)
				.unwrap()
				.out_commands
				.iter()
				.map(|c| &c.command)
				.map(|c| match c {
					BothDirectionCommand::S2CWithCreator(c) => Some(c.clone()),
					BothDirectionCommand::C2S(_) => None,
				})
				.flatten()
				.collect()
		}

		pub fn clear_user_out_commands(&mut self, user_id: RoomMemberId) {
			self.get_user_mut(user_id).unwrap().out_commands.clear();
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

		assert!(
			matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_1)
		);
		assert!(
			matches!(room.out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_2)
		);
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
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, GameObjectOwner::Room)));
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
		let user_id = room.register_user(user_template);
		room.execute_commands(user_id, &[]);
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, GameObjectOwner::User(user_id))));
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
		let user1_id = room.register_user(user1_template);
		let user2_id = room.register_user(user2_template);
		room.execute_commands(user1_id, &[]);
		room.execute_commands(
			user1_id,
			vec![CommandWithChannel {
				channel: Channel::ReliableUnordered,
				both_direction_command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			}]
			.as_slice(),
		);

		let user1 = room.get_user_mut(user1_id).unwrap();
		assert_eq!(
			user1.out_commands.pop_front().unwrap().command.get_object_id().unwrap(),
			&GameObjectId::new(object1_template.id, GameObjectOwner::User(user1_id))
		);
		user1.out_commands.clear();

		room.execute_commands(user2_id, &[]);
		let user1 = room.get_user_mut(user1_id).unwrap();
		assert_eq!(
			user1.out_commands.pop_front().unwrap().command.get_object_id().unwrap(),
			&GameObjectId::new(object2_template.id, GameObjectOwner::User(user2_id))
		);
	}

	pub fn from_vec(vec: Vec<u8>) -> heapless::Vec<u8, 256> {
		let mut result = heapless::Vec::new();
		result.extend_from_slice(vec.as_slice()).unwrap();
		result
	}

	#[test]
	pub fn should_keep_order_object() {
		let (template, user_template) = create_template();
		let mut room = Room::from_template(template);
		room.register_user(user_template);
		room.insert_object(GameObject {
			id: GameObjectId::new(100, GameObjectOwner::Room),
			template: 0,
			access_groups: Default::default(),
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			compare_and_set_owners: Default::default(),
			structures: Default::default(),
		});

		room.insert_object(GameObject {
			id: GameObjectId::new(5, GameObjectOwner::Room),
			template: 0,
			access_groups: Default::default(),
			created: false,
			longs: Default::default(),
			floats: Default::default(),
			compare_and_set_owners: Default::default(),
			structures: Default::default(),
		});

		room.insert_object(GameObject::new(GameObjectId::new(200, GameObjectOwner::Room)));

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{}{}", order, o.id.id);
		});
		assert_eq!(order, "1005200");

		room.delete_object(&GameObjectId::new(100, GameObjectOwner::Room));

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
		let user2_id = room.register_user(user2);
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
