use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;

use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::types::unload::DeleteGameObjectCommand;
use cheetah_matches_relay_common::constants::{FieldId, GameObjectTemplateId};
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, ChannelGroup, CommandWithChannel};
use cheetah_matches_relay_common::protocol::frame::channel::ChannelType;
#[cfg(test)]
use cheetah_matches_relay_common::room::access::AccessGroups;
use cheetah_matches_relay_common::room::object::GameObjectId;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::debug::tracer::CommandTracerSessions;
use crate::room::command::long::reset_all_compare_and_set;
use crate::room::command::{execute, ServerCommandError};
use crate::room::object::{CreateCommandsCollector, GameObject, S2CommandWithFieldInfo};
use crate::room::template::config::{MemberTemplate, RoomTemplate};
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
	pub template_name: String,
	pub permission_manager: Rc<RefCell<PermissionManager>>,
	pub members: HashMap<RoomMemberId, Member, FnvBuildHasher>,
	pub objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ChannelType>,
	current_member_id: Option<RoomMemberId>,
	pub user_id_generator: RoomMemberId,
	pub command_trace_session: Rc<RefCell<CommandTracerSessions>>,
	tmp_command_collector: Rc<RefCell<Vec<(GameObjectTemplateId, CreateCommandsCollector)>>>,

	#[cfg(test)]
	object_id_generator: u32,
	#[cfg(test)]
	///
	/// Исходящие команды, без проверки на прав доступа, наличия пользователей и так далее
	///
	pub out_commands: std::collections::VecDeque<(AccessGroups, S2CCommand)>,
}

#[derive(Debug)]
pub struct Member {
	pub id: RoomMemberId,
	pub connected: bool,
	pub attached: bool,
	pub template: MemberTemplate,
	pub compare_and_sets_cleaners: heapless::FnvIndexMap<(GameObjectId, FieldId), i64, 256>,
	pub out_commands: heapless::Vec<CommandWithChannelType, 64>,
}

impl Room {
	pub fn new(id: RoomId, template: RoomTemplate) -> Self {
		let mut room = Room {
			id,
			members: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			current_member_id: Default::default(),
			permission_manager: Rc::new(RefCell::new(PermissionManager::new(&template.permissions))),
			#[cfg(test)]
			object_id_generator: 0,
			#[cfg(test)]
			out_commands: Default::default(),
			user_id_generator: 0,
			command_trace_session: Default::default(),
			tmp_command_collector: Rc::new(RefCell::new(Vec::with_capacity(100))),
			template_name: template.name.clone(),
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
		F: FnMut(&RoomMemberId, &[CommandWithChannelType]),
	{
		for (user_id, user) in self.members.iter_mut() {
			let commands = user.out_commands.as_slice();
			collector(user_id, commands);
			user.out_commands.clear();
		}
	}

	///
	/// Обработать входящие команды
	///
	pub fn execute_commands(&mut self, user_id: RoomMemberId, commands: &[CommandWithChannel]) {
		let user = self.members.get_mut(&user_id);
		match user {
			None => {
				tracing::error!("[room({:?})] user({:?}) not found for input frame", self.id, user_id);
			}
			Some(user) => {
				self.current_member_id.replace(user_id);

				let connected_now = !user.connected;
				user.connected = true;

				if connected_now {
					self.current_channel.replace(ChannelType::ReliableSequence(ChannelGroup(0)));
					let user_id = user.id;
					let template = user.template.clone();
					if let Err(e) = self.on_user_connect(user_id, template) {
						e.log_error(self.id, user_id);
						return;
					}
				}
			}
		}

		let tracer = self.command_trace_session.clone();
		for command_with_channel in commands {
			match &command_with_channel.both_direction_command {
				BothDirectionCommand::C2S(command) => {
					self.current_channel.replace(From::from(&command_with_channel.channel));
					tracer.borrow_mut().collect_c2s(&self.objects, user_id, command);
					match execute(command, self, user_id) {
						Ok(_) => {}
						Err(e) => {
							e.log_error_with_command(command, self.id, user_id);
						}
					}
				}
				_ => {
					tracing::error!("[room({:?})] receive unsupported command {:?}", self.id, command_with_channel)
				}
			}
		}

		self.current_member_id = None;
		self.current_channel = None;
	}

	pub fn register_member(&mut self, template: MemberTemplate) -> RoomMemberId {
		self.user_id_generator += 1;
		let user_id = self.user_id_generator;
		let user = Member {
			id: user_id,
			connected: false,
			attached: false,
			template,
			compare_and_sets_cleaners: Default::default(),
			out_commands: Default::default(),
		};
		self.members.insert(user_id, user);
		user_id
	}

	pub fn get_member(&self, member_id: &RoomMemberId) -> Result<&Member, ServerCommandError> {
		self.members
			.get(member_id)
			.ok_or_else(|| ServerCommandError::MemberNotFound(member_id.clone()))
	}

	pub fn get_member_mut(&mut self, member_id: &RoomMemberId) -> Result<&mut Member, ServerCommandError> {
		self.members
			.get_mut(member_id)
			.ok_or_else(|| ServerCommandError::MemberNotFound(member_id.clone()))
	}

	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_user(&mut self, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
		tracing::info!("[room({:?})] disconnect user({:?})", self.id, user_id);
		self.current_member_id.replace(user_id);
		match self.members.remove(&user_id) {
			None => {}
			Some(user) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let GameObjectOwner::Member(owner) = o.id.owner {
						if owner == user.id && !o.keep_after_owner_exit {
							objects.push(o.id.clone());
						}
					}
				});

				for id in objects {
					self.delete_object(&id)?;
				}
				reset_all_compare_and_set(self, user.id, &user.compare_and_sets_cleaners)?;
			}
		};
		Ok(())
	}

	pub fn insert_object(&mut self, object: GameObject) {
		self.objects.insert(object.id.clone(), object);
	}

	pub fn get_object_mut(&mut self, object_id: &GameObjectId) -> Result<&mut GameObject, ServerCommandError> {
		self.objects
			.get_mut(object_id)
			.ok_or_else(|| ServerCommandError::GameObjectNotFound {
				object_id: object_id.clone(),
			})
	}

	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}

	pub fn delete_object(&mut self, object_id: &GameObjectId) -> Result<GameObject, ServerCommandError> {
		let current_user = self.current_member_id;
		match self.objects.shift_remove(object_id) {
			None => Err(ServerCommandError::GameObjectNotFound {
				object_id: object_id.clone(),
			}),
			Some(object) => {
				self.send_to_members(
					object.access_groups,
					object.template_id,
					&[S2CommandWithFieldInfo {
						field: None,
						command: S2CCommand::Delete(DeleteGameObjectCommand {
							object_id: object.id.clone(),
						}),
					}],
					|user| {
						if let Some(user_id) = current_user {
							user.id != user_id
						} else {
							true
						}
					},
				)?;
				Ok(object)
			}
		}
	}

	pub fn process_objects(&self, f: &mut dyn FnMut(&GameObject)) {
		self.objects.iter().for_each(|(_, o)| f(o));
	}

	fn on_user_connect(&mut self, user_id: RoomMemberId, template: MemberTemplate) -> Result<(), ServerCommandError> {
		for object_template in template.objects {
			let object = object_template.create_user_game_object(user_id);
			let mut commands = CreateCommandsCollector::new();
			object.collect_create_commands(&mut commands);
			let template = object.template_id;
			let access_groups = object.access_groups;
			self.send_to_members(access_groups, template, commands.as_slice(), |_user| true)?;
			self.insert_object(object);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use cheetah_matches_relay_common::commands::c2s::C2SCommand;
	use cheetah_matches_relay_common::commands::s2c::S2CCommand::SetLong;
	use cheetah_matches_relay_common::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use cheetah_matches_relay_common::commands::types::long::SetLongCommand;
	use cheetah_matches_relay_common::commands::FieldType;
	use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
	use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
	use cheetah_matches_relay_common::protocol::frame::channel::{Channel, ChannelType};
	use cheetah_matches_relay_common::room::access::AccessGroups;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;
	use cheetah_matches_relay_common::room::RoomMemberId;

	use crate::room::object::GameObject;
	use crate::room::template::config::{GameObjectTemplate, MemberTemplate, Permission, RoomTemplate};
	use crate::room::{Room, ServerCommandError};

	impl Default for Room {
		fn default() -> Self {
			Room::new(0, RoomTemplate::default())
		}
	}

	impl Room {
		pub fn from_template(template: RoomTemplate) -> Self {
			Room::new(0, template)
		}

		pub fn test_create_object(
			&mut self,
			owner: RoomMemberId,
			access_groups: AccessGroups,
			keep_after_owner_exit: bool,
		) -> &mut GameObject {
			self.object_id_generator += 1;
			let id = GameObjectId::new(self.object_id_generator, GameObjectOwner::Member(owner));
			let object = GameObject::new(id.clone(), 0, access_groups, false, keep_after_owner_exit);
			self.insert_object(object);
			self.get_object_mut(&id).unwrap()
		}

		pub fn test_mark_as_connected(&mut self, user_id: RoomMemberId) -> Result<(), ServerCommandError> {
			let member = self.get_member_mut(&user_id)?;
			member.connected = true;
			member.attached = true;
			Ok(())
		}

		pub fn test_get_user_out_commands(&self, user_id: RoomMemberId) -> VecDeque<S2CCommand> {
			self.get_member(&user_id)
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

		pub fn test_get_user_out_commands_with_meta(&self, user_id: RoomMemberId) -> VecDeque<S2CCommandWithCreator> {
			self.get_member(&user_id)
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

		pub fn test_clear_user_out_commands(&mut self, user_id: RoomMemberId) {
			self.get_member_mut(&user_id).unwrap().out_commands.clear();
		}
	}

	#[test]
	fn should_remove_objects_when_disconnect() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b111);
		let mut room = Room::from_template(template);
		let user_a = room.register_member(MemberTemplate::stub(access_groups));
		let user_b = room.register_member(MemberTemplate::stub(access_groups));
		let object_a_1 = room.test_create_object(user_a, access_groups, false).id.clone();
		let object_a_2 = room.test_create_object(user_a, access_groups, false).id.clone();
		let object_a_3_with_keep_flag = room.test_create_object(user_a, access_groups, true).id.clone();
		let object_b_1 = room.test_create_object(user_b, access_groups, false).id.clone();
		let object_b_2 = room.test_create_object(user_b, access_groups, false).id.clone();

		room.out_commands.clear();
		room.disconnect_user(user_a).unwrap();

		assert!(!room.contains_object(&object_a_1));
		assert!(!room.contains_object(&object_a_2));
		assert!(room.contains_object(&object_a_3_with_keep_flag));

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
		let user_template = MemberTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: vec![object_template.clone()],
		};
		let mut room = Room::from_template(template);
		let user_id = room.register_member(user_template);
		room.execute_commands(user_id, &[]);
		assert!(room
			.objects
			.contains_key(&GameObjectId::new(object_template.id, GameObjectOwner::Member(user_id))));
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
		let user1_template = MemberTemplate {
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
		let user2_template = MemberTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: vec![object2_template.clone()],
		};

		let mut room = Room::from_template(template);
		let user1_id = room.register_member(user1_template);
		let user2_id = room.register_member(user2_template);
		room.execute_commands(user1_id, &[]);
		room.execute_commands(
			user1_id,
			vec![CommandWithChannel {
				channel: Channel::ReliableUnordered,
				both_direction_command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			}]
			.as_slice(),
		);

		let user1 = room.get_member_mut(&user1_id).unwrap();
		assert_eq!(
			user1.out_commands[0].command.get_object_id().unwrap(),
			&GameObjectId::new(object1_template.id, GameObjectOwner::Member(user1_id))
		);
		user1.out_commands.clear();

		room.execute_commands(user2_id, &[]);
		let user1 = room.get_member_mut(&user1_id).unwrap();
		assert_eq!(
			user1.out_commands[1].command.get_object_id().unwrap(),
			&GameObjectId::new(object2_template.id, GameObjectOwner::Member(user2_id))
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
		room.register_member(user_template);
		room.insert_object(GameObject::new(
			GameObjectId::new(100, GameObjectOwner::Room),
			0,
			Default::default(),
			false,
			false,
		));

		room.insert_object(GameObject::new(
			GameObjectId::new(5, GameObjectOwner::Room),
			0,
			Default::default(),
			false,
			false,
		));

		room.insert_object(GameObject::new(
			GameObjectId::new(200, GameObjectOwner::Room),
			0,
			Default::default(),
			false,
			false,
		));

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{}{}", order, o.id.id);
		});
		assert_eq!(order, "1005200");

		room.delete_object(&GameObjectId::new(100, GameObjectOwner::Room)).unwrap();

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

		let mut user1_template = MemberTemplate::stub(groups);
		let object1_template = user1_template.configure_object(1, 100, groups);
		let allow_field_id = 5;
		let deny_field_id = 10;
		object1_template.fields.longs.insert(allow_field_id, 555);
		object1_template.fields.longs.insert(deny_field_id, 111);
		template
			.permissions
			.set_permission(100, &deny_field_id, FieldType::Long, &groups, Permission::Deny);

		let mut room = Room::from_template(template);
		let user1_id = room.register_member(user1_template.clone());
		let user2 = MemberTemplate::stub(groups);
		let user2_id = room.register_member(user2);
		room.test_mark_as_connected(user2_id).unwrap();
		room.on_user_connect(user1_id, user1_template.clone()).unwrap();

		let commands = room.test_get_user_out_commands(user2_id);

		assert!(matches!(commands.get(0), Some(S2CCommand::Create(_))));
		assert!(matches!(commands.get(1), Some(S2CCommand::SetLong(command)) if command.field_id == allow_field_id));
		assert!(matches!(commands.get(2), Some(S2CCommand::Created(_))));
	}

	#[test]
	pub fn should_clear_out_commands_after_collect() {
		let mut room = Room::default();
		let member_template = MemberTemplate::stub(AccessGroups(8));
		let member_id = room.register_member(member_template);
		room.test_mark_as_connected(member_id.clone()).unwrap();
		let member = room.get_member_mut(&member_id).unwrap();
		member
			.out_commands
			.push(CommandWithChannelType {
				channel_type: ChannelType::ReliableUnordered,
				command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
					command: SetLong(SetLongCommand {
						object_id: Default::default(),
						field_id: 0,
						value: 0,
					}),
					creator: 0,
				}),
			})
			.unwrap();
		room.collect_out_commands(|_, _| {});
		let member = room.get_member(&member_id).unwrap();
		assert!(member.out_commands.is_empty());
	}

	pub fn create_template() -> (RoomTemplate, MemberTemplate) {
		let template = RoomTemplate::default();
		let user_template = MemberTemplate {
			private_key: Default::default(),
			groups: AccessGroups(55),
			objects: Default::default(),
		};
		(template, user_template)
	}
}
