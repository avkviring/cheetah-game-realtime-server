use std::collections::HashMap;
use std::fmt::Debug;
use std::slice;
use std::time::{Duration, Instant};

use fnv::{FnvBuildHasher, FnvHashMap};
use indexmap::map::IndexMap;
use serde::{Deserialize, Serialize};

use cheetah_common::commands::guarantees::{ChannelGroup, ReliabilityGuarantees};
use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithMeta};
use cheetah_common::commands::types::delete::DeleteGameObjectCommand;
use cheetah_common::commands::types::member::{MemberConnected, MemberDisconnected};
use cheetah_common::commands::{BothDirectionCommand, CommandWithChannelType, CommandWithReliabilityGuarantees};
use cheetah_common::room::access::AccessGroups;
use cheetah_common::room::buffer::Buffer;
use cheetah_common::room::object::GameObjectId;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_protocol::{RoomId, RoomMemberId};
use member::RoomMember;

use crate::room::command::{execute, ServerCommandError};
use crate::room::object::{GameObject, S2CCommandsCollector};
use crate::room::template::config::{MemberTemplate, RoomTemplate};

pub mod action;
pub mod command;
pub mod member;
pub mod object;
pub mod sender;
pub mod template;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
	pub id: RoomId,
	pub template_name: String,
	pub members: HashMap<RoomMemberId, RoomMember, FnvBuildHasher>,
	pub(crate) objects: IndexMap<GameObjectId, GameObject, FnvBuildHasher>,
	current_channel: Option<ReliabilityGuarantees>,
	pub member_id_generator: RoomMemberId,
	pub room_object_id_generator: u32,
	objects_singleton_key: HashMap<Buffer, GameObjectId, FnvBuildHasher>,

	#[cfg(test)]
	test_object_id_generator: u32,
	#[cfg(test)]
	///
	/// Исходящие команды, без проверки на прав доступа, наличия пользователей и так далее
	///
	pub test_out_commands: std::collections::VecDeque<(AccessGroups, S2CCommand)>,
}
impl Room {
	pub fn new(id: RoomId, template: RoomTemplate) -> Self {
		let mut room = Room {
			id,
			members: FnvHashMap::default(),
			objects: Default::default(),
			current_channel: Default::default(),
			#[cfg(test)]
			test_object_id_generator: 0,
			#[cfg(test)]
			test_out_commands: Default::default(),
			member_id_generator: 0,
			room_object_id_generator: 65536,
			template_name: template.name.clone(),
			objects_singleton_key: Default::default(),
		};

		template.objects.into_iter().for_each(|object| {
			let game_object: GameObject = object.to_root_game_object();
			room.insert_object(game_object);
		});

		room
	}

	pub(crate) fn has_object_singleton_key(&self, value: &Buffer) -> bool {
		match self.objects_singleton_key.get(value) {
			None => false,
			Some(object_id) => self.objects.contains_key(object_id),
		}
	}

	pub fn set_singleton_key(&mut self, unique_key: Buffer, object_id: GameObjectId) {
		self.objects_singleton_key.insert(unique_key, object_id);
	}

	///
	/// Получить команды для отправки в сеть
	///
	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomMemberId, &[CommandWithChannelType]),
	{
		for (member_id, member) in &mut self.members {
			let commands = member.out_commands.as_slice();
			collector(member_id, commands);
			member.out_commands.clear();
		}
	}

	///
	/// Обработать входящие команды.
	///
	/// Пользователь должен быть добавлен в комнату через [`Self::register_member`] до выполнения команд.
	///
	/// Если комната не сконфигурирована [`Self::is_ready`] то команды не-суперпользователей будут игнорироваться.
	///
	/// Если в комнате настроен форвардинг [`Self::should_forward`],
	/// то команды не-суперпользователей будут перенаправлены суперпользователям вместо выполнения.
	///
	pub fn execute_commands(&mut self, member_id: RoomMemberId, commands: &[CommandWithReliabilityGuarantees]) {
		if let Some(member) = self.members.get(&member_id) {
			if !member.connected {
				if let Err(e) = self.connect_member(member_id) {
					e.log_error(self.id, member_id);
					return;
				}
			}
		} else {
			tracing::error!("[room({:?})] member({:?}) not found for input frame", self.id, member_id);
			self.current_channel = None;
			return;
		}

		for command_with_channel in commands {
			match &command_with_channel.command {
				BothDirectionCommand::C2S(command) => {
					self.current_channel.replace(From::from(&command_with_channel.reliability_guarantees));

					let instant = Instant::now();
					match execute(command, self, member_id) {
						Ok(_) => {}
						Err(e) => {
							e.log_command_execute_error(command, self.id, member_id);
						}
					}
					if instant.elapsed() > Duration::from_millis(100) {
						tracing::error!("Slow command {:?}", command);
					}
				}
				BothDirectionCommand::S2CWithCreator(_) => {
					tracing::error!("[room({:?})] receive unsupported command {:?}", self.id, command_with_channel);
				}
			}
		}

		self.current_channel = None;
	}

	fn connect_member(&mut self, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		self.current_channel.replace(ReliabilityGuarantees::ReliableSequence(ChannelGroup(0)));
		let member = self.members.get(&member_id).ok_or(ServerCommandError::MemberNotFound(member_id))?;
		let template = member.template.clone();
		if let Err(e) = self.on_member_connect(member_id, template) {
			self.current_channel = None;
			return Err(e);
		}

		let member = self.members.get_mut(&member_id).ok_or(ServerCommandError::MemberNotFound(member_id))?;
		member.connected = true;
		Ok(())
	}

	pub fn register_member(&mut self, template: MemberTemplate) -> RoomMemberId {
		self.member_id_generator += 1;
		let member_id = self.member_id_generator;
		let member = RoomMember {
			id: member_id,
			connected: false,
			attached: false,
			template,
			out_commands: Default::default(),
		};
		self.members.insert(member_id, member);
		tracing::info!("[room({:?})] register member({:?})", self.id, member_id);
		member_id
	}

	pub fn get_member(&self, member_id: &RoomMemberId) -> Result<&RoomMember, ServerCommandError> {
		self.members.get(member_id).ok_or(ServerCommandError::MemberNotFound(*member_id))
	}

	pub fn get_member_mut(&mut self, member_id: &RoomMemberId) -> Result<&mut RoomMember, ServerCommandError> {
		self.members.get_mut(member_id).ok_or(ServerCommandError::MemberNotFound(*member_id))
	}

	///
	/// Связь с пользователям разорвана
	/// удаляем все созданные им объекты с уведомлением других пользователей
	///
	pub fn disconnect_member(&mut self, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
		tracing::info!("[room({:?})] disconnect member({:?})", self.id, member_id);
		match self.members.remove(&member_id) {
			None => {}
			Some(member) => {
				let mut objects = Vec::new();
				self.process_objects(&mut |o| {
					if let GameObjectOwner::Member(owner) = o.id.get_owner() {
						if owner == member.id {
							objects.push(o.id);
						}
					}
				});

				for id in objects {
					self.delete_object(id, member_id)?;
				}
			}
		};

		let s2c = S2CCommandWithMeta {
			field: None,
			creator: member_id,
			command: S2CCommand::MemberDisconnected(MemberDisconnected { member_id }),
		};
		self.send_to_members(AccessGroups::super_member_group(), slice::from_ref(&s2c), |member| member.id != member_id)?;

		Ok(())
	}

	pub fn insert_object(&mut self, object: GameObject) {
		self.objects.insert(object.id, object);
	}

	pub fn get_object(&self, object_id: GameObjectId) -> Result<&GameObject, ServerCommandError> {
		self.objects.get(&object_id).ok_or(ServerCommandError::GameObjectNotFound { object_id })
	}

	pub fn get_object_mut(&mut self, object_id: GameObjectId) -> Result<&mut GameObject, ServerCommandError> {
		self.objects.get_mut(&object_id).ok_or(ServerCommandError::GameObjectNotFound { object_id })
	}

	#[must_use]
	pub fn contains_object(&self, object_id: &GameObjectId) -> bool {
		self.objects.contains_key(object_id)
	}

	pub fn delete_object(&mut self, object_id: GameObjectId, member_id: RoomMemberId) -> Result<GameObject, ServerCommandError> {
		match self.objects.shift_remove(&object_id) {
			None => Err(ServerCommandError::GameObjectNotFound { object_id }),
			Some(object) => {
				if object.created {
					self.send_to_members(
						object.access_groups,
						&[S2CCommandWithMeta {
							field: None,
							creator: member_id,
							command: S2CCommand::Delete(DeleteGameObjectCommand { object_id: object.id }),
						}],
						|member| member.id != member_id,
					)?;
				}
				Ok(object)
			}
		}
	}

	pub fn process_objects(&self, f: &mut dyn FnMut(&GameObject)) {
		self.objects.iter().for_each(|(_, o)| f(o));
	}

	fn on_member_connect(&mut self, member_id: RoomMemberId, template: MemberTemplate) -> Result<(), ServerCommandError> {
		for object_template in template.objects {
			let object = object_template.create_member_game_object(member_id);
			let mut commands = S2CCommandsCollector::new();
			object.collect_create_commands(&mut commands, member_id);
			let access_groups = object.access_groups;
			self.send_to_members(access_groups, commands.as_slice(), |_member_id| true)?;
			self.insert_object(object);
		}

		let s2c = S2CCommandWithMeta {
			field: None,
			creator: member_id,
			command: S2CCommand::MemberConnected(MemberConnected { member_id }),
		};
		self.send_to_members(AccessGroups::super_member_group(), slice::from_ref(&s2c), |other_member| other_member.id != member_id)?;

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use cheetah_common::commands::c2s::C2SCommand;
	use cheetah_common::commands::guarantees::{ReliabilityGuarantees, ReliabilityGuaranteesChannel};
	use cheetah_common::commands::s2c::{S2CCommand, S2CCommandWithCreator};
	use cheetah_common::commands::types::long::SetLongCommand;
	use cheetah_common::commands::types::member::{MemberConnected, MemberDisconnected};
	use cheetah_common::commands::{BothDirectionCommand, CommandWithChannelType, CommandWithReliabilityGuarantees};
	use cheetah_common::room::access::AccessGroups;
	use cheetah_common::room::buffer::Buffer;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;
	use cheetah_protocol::RoomMemberId;

	use crate::room::object::GameObject;
	use crate::room::template::config::{GameObjectTemplate, MemberTemplate, RoomTemplate};
	use crate::room::{Room, ServerCommandError};

	impl Default for Room {
		fn default() -> Self {
			Room::new(0, RoomTemplate::default())
		}
	}

	impl Room {
		#[must_use]
		pub fn from_template(template: RoomTemplate) -> Self {
			Room::new(0, template)
		}

		pub fn test_create_object_with_not_created_state(&mut self, owner: GameObjectOwner, access_groups: AccessGroups) -> &mut GameObject {
			self.test_do_create_object(owner, access_groups, false)
		}

		pub fn test_create_object_with_created_state(&mut self, owner: GameObjectOwner, access_groups: AccessGroups) -> &mut GameObject {
			self.test_do_create_object(owner, access_groups, true)
		}

		fn test_do_create_object(&mut self, owner: GameObjectOwner, access_groups: AccessGroups, created: bool) -> &mut GameObject {
			self.test_object_id_generator += 1;
			let id = GameObjectId::new(self.test_object_id_generator, owner);
			let mut object = GameObject::new(id, 0, access_groups, false);
			object.created = created;
			self.insert_object(object);
			self.get_object_mut(id).unwrap()
		}

		pub fn mark_as_connected_in_test(&mut self, member_id: RoomMemberId) -> Result<(), ServerCommandError> {
			let member = self.get_member_mut(&member_id)?;
			member.connected = true;
			member.attached = true;
			Ok(())
		}

		#[must_use]
		pub fn get_member_out_commands_for_test(&self, member_id: RoomMemberId) -> VecDeque<S2CCommand> {
			self.get_member(&member_id)
				.unwrap()
				.out_commands
				.iter()
				.map(|c| &c.command)
				.filter_map(|c| match c {
					BothDirectionCommand::S2CWithCreator(c) => Some(c.command.clone()),
					BothDirectionCommand::C2S(_) => None,
				})
				.collect()
		}

		#[must_use]
		pub fn test_get_member_out_commands_with_meta(&self, member_id: RoomMemberId) -> VecDeque<S2CCommandWithCreator> {
			self.get_member(&member_id)
				.unwrap()
				.out_commands
				.iter()
				.map(|c| &c.command)
				.filter_map(|c| match c {
					BothDirectionCommand::S2CWithCreator(c) => Some(c.clone()),
					BothDirectionCommand::C2S(_) => None,
				})
				.collect()
		}

		pub fn test_clear_member_out_commands(&mut self, member_id: RoomMemberId) {
			self.get_member_mut(&member_id).unwrap().out_commands.clear();
		}
	}

	#[test]
	fn should_remove_objects_when_disconnect() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(0b111);
		let mut room = Room::from_template(template);
		let member_a = room.register_member(MemberTemplate::stub(access_groups));
		let member_b = room.register_member(MemberTemplate::stub(access_groups));
		let object_a_1 = room.test_create_object_with_created_state(GameObjectOwner::Member(member_a), access_groups).id;
		let object_a_2 = room.test_create_object_with_created_state(GameObjectOwner::Member(member_a), access_groups).id;
		let object_b_1 = room.test_create_object_with_created_state(GameObjectOwner::Member(member_b), access_groups).id;
		let object_b_2 = room.test_create_object_with_created_state(GameObjectOwner::Member(member_b), access_groups).id;

		room.test_out_commands.clear();
		room.disconnect_member(member_a).unwrap();

		assert!(!room.contains_object(&object_a_1));
		assert!(!room.contains_object(&object_a_2));

		assert!(room.contains_object(&object_b_1));
		assert!(room.contains_object(&object_b_2));

		assert!(matches!(room.test_out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_1));
		assert!(matches!(room.test_out_commands.pop_back(), Some((..,S2CCommand::Delete(command))) if command.object_id == object_a_2));
	}

	#[test]
	fn should_create_object_from_config() {
		let mut template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			groups: Default::default(),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		template.objects = vec![object_template.clone()];

		let room = Room::from_template(template);
		assert!(room.objects.contains_key(&GameObjectId::new(object_template.id, GameObjectOwner::Room)));
	}

	#[test]
	fn should_create_object_from_config_for_member() {
		let template = RoomTemplate::default();
		let object_template = GameObjectTemplate {
			id: 155,
			template: 5,
			groups: AccessGroups(55),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		let member_template = MemberTemplate::new_member(AccessGroups(55), vec![object_template.clone()]);
		let mut room = Room::from_template(template);
		let member_id = room.register_member(member_template);
		room.execute_commands(member_id, &[]);
		assert!(room.objects.contains_key(&GameObjectId::new(object_template.id, GameObjectOwner::Member(member_id))));
	}

	///
	/// Пользовательские объекты из шаблона должны загружаться на первый клиент при входе второго
	///
	#[test]
	fn should_load_member_object_when_connect_other_member() {
		let template = RoomTemplate::default();
		let object1_template = GameObjectTemplate {
			id: 100,
			template: 5,
			groups: AccessGroups(55),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		let member1_template = MemberTemplate::new_member(AccessGroups(55), vec![object1_template.clone()]);

		let object2_template = GameObjectTemplate {
			id: 200,
			template: 5,
			groups: AccessGroups(55),
			longs: Default::default(),
			doubles: Default::default(),
			structures: Default::default(),
		};
		let member2_template = MemberTemplate::new_member(AccessGroups(55), vec![object2_template.clone()]);

		let mut room = Room::from_template(template);
		let member1_id = room.register_member(member1_template);
		let member2_id = room.register_member(member2_template);
		room.execute_commands(member1_id, &[]);
		room.execute_commands(
			member1_id,
			vec![CommandWithReliabilityGuarantees {
				reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
				command: BothDirectionCommand::C2S(C2SCommand::AttachToRoom),
			}]
			.as_slice(),
		);

		let member1 = room.get_member_mut(&member1_id).unwrap();
		assert_eq!(
			member1.out_commands[0].command.get_object_id().unwrap(),
			GameObjectId::new(object1_template.id, GameObjectOwner::Member(member1_id))
		);
		member1.out_commands.clear();

		room.execute_commands(member2_id, &[]);
		let member1 = room.get_member_mut(&member1_id).unwrap();
		assert_eq!(
			member1.out_commands[1].command.get_object_id().unwrap(),
			GameObjectId::new(object2_template.id, GameObjectOwner::Member(member2_id))
		);
	}

	#[test]
	pub(crate) fn should_keep_order_object() {
		let (template, member_template) = create_template();
		let mut room = Room::from_template(template);
		room.register_member(member_template);
		room.insert_object(GameObject::new(GameObjectId::new(100, GameObjectOwner::Room), 0, Default::default(), false));

		room.insert_object(GameObject::new(GameObjectId::new(5, GameObjectOwner::Room), 0, Default::default(), false));

		room.insert_object(GameObject::new(GameObjectId::new(200, GameObjectOwner::Room), 0, Default::default(), false));

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{order}{}", o.id.id);
		});
		assert_eq!(order, "1005200");

		room.delete_object(GameObjectId::new(100, GameObjectOwner::Room), u16::MAX).unwrap();

		let mut order = String::new();
		room.objects.values().for_each(|o| {
			order = format!("{order}{}", o.id.id);
		});
		assert_eq!(order, "5200");
	}

	#[test]
	pub(crate) fn should_clear_out_commands_after_collect() {
		let mut room = Room::default();
		let member_template = MemberTemplate::stub(AccessGroups(8));
		let member_id = room.register_member(member_template);
		room.mark_as_connected_in_test(member_id).unwrap();
		let member = room.get_member_mut(&member_id).unwrap();
		member.out_commands.push(CommandWithChannelType {
			channel_type: ReliabilityGuarantees::ReliableUnordered,
			command: BothDirectionCommand::S2CWithCreator(S2CCommandWithCreator {
				command: S2CCommand::SetLong(SetLongCommand {
					object_id: Default::default(),
					field_id: 0,
					value: 0,
				}),
				creator: 0,
			}),
		});
		room.collect_out_commands(|_, _| {});
		let member = room.get_member(&member_id).unwrap();
		assert!(member.out_commands.is_empty());
	}

	#[test]
	fn should_check_singleton_key() {
		let mut room = Room::default();
		let object = room.test_create_object_with_not_created_state(GameObjectOwner::Room, AccessGroups(7));
		let object_id = object.id;
		let unique_key = Buffer::from([1, 2, 3, 4].as_slice());
		room.set_singleton_key(unique_key.clone(), object_id);
		assert!(room.has_object_singleton_key(&unique_key));
		room.delete_object(object_id, u16::MAX).unwrap();
		assert!(!room.has_object_singleton_key(&unique_key));
	}

	#[test]
	fn should_send_member_connected() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		room.mark_as_connected_in_test(member_1).unwrap();

		let member_2 = room.register_member(MemberTemplate::stub(access_groups));
		room.mark_as_connected_in_test(member_2).unwrap();
		room.connect_member(member_2).unwrap();

		assert_eq!(S2CCommand::MemberConnected(MemberConnected { member_id: member_2 }), room.get_member_out_commands_for_test(member_1)[0]);
	}

	#[test]
	fn should_send_member_disconnect() {
		let template = RoomTemplate::default();
		let access_groups = AccessGroups(10);
		let mut room = Room::from_template(template);
		let member_1 = room.register_member(MemberTemplate::stub(access_groups));
		room.mark_as_connected_in_test(member_1).unwrap();
		room.connect_member(member_1).unwrap();

		let member_2 = room.register_member(MemberTemplate::stub(access_groups));
		room.mark_as_connected_in_test(member_2).unwrap();
		room.connect_member(member_2).unwrap();
		room.disconnect_member(member_2).unwrap();

		assert_eq!(
			S2CCommand::MemberDisconnected(MemberDisconnected { member_id: member_2 }),
			room.get_member_out_commands_for_test(member_1)[1]
		);
	}

	pub(crate) fn create_template() -> (RoomTemplate, MemberTemplate) {
		let template = RoomTemplate::default();
		let member_template = MemberTemplate::new_member(AccessGroups(55), Default::default());
		(template, member_template)
	}
}
