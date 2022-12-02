use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fnv::{FnvBuildHasher, FnvHashSet};
use thiserror::Error;

use cheetah_matches_realtime_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_realtime_common::protocol::frame::applications::CommandWithChannel;
use cheetah_matches_realtime_common::protocol::others::member_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::Room;
use crate::server::measurers::{MeasureStringId, Measurers};

#[derive(Default)]
pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	measurers: Rc<RefCell<Measurers>>,
	plugin_names: FnvHashSet<String>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("room not found {0}")]
pub struct RoomNotFoundError(pub RoomId);

impl Rooms {
	pub fn new(measurers: Rc<RefCell<Measurers>>, plugin_names: FnvHashSet<String>) -> Self {
		Self {
			room_by_id: Default::default(),
			room_id_generator: 0,
			measurers,
			plugin_names,
		}
	}

	pub fn create_room(&mut self, template: RoomTemplate) -> RoomId {
		self.room_id_generator += 1;
		self.measurers.borrow_mut().on_create_room(&template.name);

		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template, Rc::clone(&self.measurers), self.plugin_names.clone());
		self.room_by_id.insert(room_id, room);
		room_id
	}

	/// удалить комнату из списка без изменений пользователей и объектов
	pub fn take_room(&mut self, room_id: &RoomId) -> Result<Room, RoomNotFoundError> {
		self.room_by_id.remove(room_id).ok_or(RoomNotFoundError(*room_id))
	}

	pub fn register_member(&mut self, room_id: RoomId, member_template: MemberTemplate) -> Result<RoomMemberId, RoomNotFoundError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Err(RoomNotFoundError(room_id)),
			Some(room) => {
				let result = Ok(room.register_member(member_template));
				if result.is_ok() {
					self.measurers.borrow_mut().on_change_member_count(&room.template_name, 1);
				}
				result
			}
		}
	}

	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomId, &RoomMemberId, &[CommandWithChannelType]),
	{
		for (room_id, room) in &mut self.room_by_id {
			let template = MeasureStringId::from(room.template_name.as_str());
			room.collect_out_commands(|member_id, commands| {
				collector(room_id, member_id, commands);
				self.measurers.borrow_mut().on_output_commands(&template, commands);
			});
		}
	}

	pub fn execute_commands(&mut self, member_and_room_id: MemberAndRoomId, commands: &[CommandWithChannel]) {
		match self.room_by_id.get_mut(&member_and_room_id.room_id) {
			None => {
				tracing::error!("[rooms] on_frame_received room({}) not found", member_and_room_id.room_id);
			}
			Some(room) => {
				let object_count = room.objects.len();
				room.execute_commands(member_and_room_id.member_id, commands);

				let mut measurers = self.measurers.borrow_mut();

				let delta_object_count: i64 = room.objects.len() as i64 - object_count as i64;
				if delta_object_count > 0 {
					measurers.on_change_object_count(&room.template_name, delta_object_count);
				}
				measurers.on_input_commands(&room.template_name, commands);
			}
		}
	}

	pub fn member_disconnected(&mut self, member_and_room_id: &MemberAndRoomId) -> Result<(), ServerCommandError> {
		match self.room_by_id.get_mut(&member_and_room_id.room_id) {
			None => Err(ServerCommandError::RoomNotFound(RoomNotFoundError(member_and_room_id.room_id))),
			Some(room) => {
				room.disconnect_member(member_and_room_id.member_id)?;
				self.measurers.borrow_mut().on_change_member_count(&room.template_name, -1);
				Ok(())
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_remove_room() {
		let mut rooms = Rooms::default();
		let room_id = rooms.create_room(RoomTemplate::default());
		let room = rooms.take_room(&room_id);
		assert!(room.is_ok(), "want room when take by room_id");
		assert_eq!(room_id, room.unwrap().id, "want taken room_id to match with room_id parameter");
		assert!(!rooms.room_by_id.contains_key(&room_id), "want room_id to be removed from rooms");
	}

	#[test]
	fn should_remove_room_room_not_found() {
		let mut rooms = Rooms::default();
		let room_id = 123;
		let room = rooms.take_room(&room_id);
		assert!(room.is_err(), "want error when take non existing room");
		assert_eq!(room_id, room.err().unwrap().0, "want the same room_id in take_room parameter and error");
	}
}
