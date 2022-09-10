use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fnv::FnvBuildHasher;
use thiserror::Error;

use cheetah_matches_realtime_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_realtime_common::protocol::frame::applications::CommandWithChannel;
use cheetah_matches_realtime_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_realtime_common::room::{RoomId, RoomMemberId};

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::Room;
use crate::server::measurers::{MeasureStringId, Measurers};

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: heapless::FnvIndexSet<RoomId, 10_000>,
	measures: Rc<RefCell<Measurers>>,
}

#[derive(Debug, Error)]
pub enum RegisterUserError {
	#[error("RoomNotFound")]
	RoomNotFound,
}

impl Rooms {
	pub fn new(measures: Rc<RefCell<Measurers>>) -> Self {
		Self {
			room_by_id: Default::default(),
			room_id_generator: 0,
			changed_rooms: Default::default(),
			measures,
		}
	}

	pub fn create_room(&mut self, template: RoomTemplate) -> RoomId {
		self.room_id_generator += 1;
		self.measures.borrow_mut().on_create_room(&template.name);

		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template, self.measures.clone());
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, member_template: MemberTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				let result = Ok(room.register_member(member_template));
				if result.is_ok() {
					self.measures.borrow_mut().on_change_member_count(&room.template_name, 1);
				}
				result
			}
		}
	}

	pub fn collect_out_commands<F>(&mut self, mut collector: F)
	where
		F: FnMut(&RoomId, &RoomMemberId, &[CommandWithChannelType]),
	{
		for room_id in self.changed_rooms.iter() {
			match self.room_by_id.get_mut(room_id) {
				None => {}
				Some(room) => {
					let room_id = room.id;
					let template = MeasureStringId::from(room.template_name.as_str());
					room.collect_out_commands(|user_id, commands| {
						collector(&room_id, user_id, commands);
						self.measures.borrow_mut().on_output_commands(&template, commands);
					});
				}
			}
		}
	}

	pub fn execute_commands(&mut self, user_and_room_id: MemberAndRoomId, commands: &[CommandWithChannel]) {
		match self.room_by_id.get_mut(&user_and_room_id.room_id) {
			None => {
				tracing::error!("[rooms] on_frame_received room({}) not found", user_and_room_id.room_id);
			}
			Some(room) => {
				let object_count = room.objects.len();
				room.execute_commands(user_and_room_id.member_id, commands);
				self.changed_rooms.insert(room.id).unwrap();

				let mut measures = self.measures.borrow_mut();
				let delta_object_count = room.objects.len() - object_count;
				if delta_object_count > 0 {
					measures.on_change_object_count(&room.template_name, delta_object_count as i64);
				}
				measures.on_input_commands(&room.template_name, commands);
			}
		}
	}

	pub fn user_disconnected(&mut self, member_and_room_id: &MemberAndRoomId) -> Result<(), ServerCommandError> {
		match self.room_by_id.get_mut(&member_and_room_id.room_id) {
			None => Err(ServerCommandError::Error(format!(
				"[rooms] room not found ({:?}) in user_disconnect",
				member_and_room_id
			))),
			Some(room) => {
				room.disconnect_user(member_and_room_id.member_id)?;
				self.measures.borrow_mut().on_change_member_count(&room.template_name, -1);
				self.changed_rooms.insert(member_and_room_id.room_id).unwrap();
				Ok(())
			}
		}
	}
}
