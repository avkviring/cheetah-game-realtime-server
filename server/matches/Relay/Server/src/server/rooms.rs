use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::CommandWithChannel;
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::Room;
use crate::server::measures::{HeaplessStatisticString, ServerMeasures};

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: heapless::FnvIndexSet<RoomId, 10_000>,
	measures: Rc<RefCell<ServerMeasures>>,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
}

impl Rooms {
	pub fn new(measures: Rc<RefCell<ServerMeasures>>) -> Self {
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
		let room = Room::new(room_id, template);
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, member_template: MemberTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				self.measures.borrow_mut().on_user_register(&room.template_name);
				Result::Ok(room.register_member(member_template))
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
					let template = HeaplessStatisticString::from(room.template_name.as_str());
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
				room.execute_commands(user_and_room_id.member_id, commands);
				self.changed_rooms.insert(room.id).unwrap();
				let template = heapless::String::<50>::from(room.template_name.as_str());
				self.measures.borrow_mut().on_input_commands(&template, commands);
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
				self.measures.borrow_mut().on_user_disconnected(&room.template_name);
				room.disconnect_user(member_and_room_id.member_id)?;
				self.changed_rooms.insert(member_and_room_id.room_id).unwrap();
				Ok(())
			}
		}
	}
}
