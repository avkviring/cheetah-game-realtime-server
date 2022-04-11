use std::collections::HashMap;

use fnv::FnvBuildHasher;
use prometheus::core::AtomicI64;

use cheetah_matches_relay_common::collections::event_collector_by_time::Int;
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::CommandWithChannel;
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};
use cheetah_microservice::prometheus::gauge::GaugeByTagMeasures;

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::Room;

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: heapless::FnvIndexSet<RoomId, 10_000>,
	prometheus_room_count: GaugeByTagMeasures<AtomicI64>,
	prometheus_user_count: GaugeByTagMeasures<AtomicI64>,
}

#[derive(Debug)]
pub enum RegisterUserError {
	RoomNotFound,
}

impl Default for Rooms {
	fn default() -> Self {
		Rooms {
			room_by_id: Default::default(),
			room_id_generator: 0,
			changed_rooms: Default::default(),
			prometheus_room_count: GaugeByTagMeasures::<AtomicI64>::new("room_count", "Room by template", "template"),
			prometheus_user_count: GaugeByTagMeasures::<AtomicI64>::new("user_count", "User count", "template"),
		}
	}
}

impl Rooms {
	pub fn create_room(&mut self, template: RoomTemplate) -> RoomId {
		self.room_id_generator += 1;
		#[cfg(not(test))]
		self.prometheus_room_count.inc(template.name.as_str());
		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template);
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, member_template: MemberTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				#[cfg(not(test))]
				self.prometheus_user_count.inc(room.template_name.as_str());
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
					room.collect_out_commands(|user_id, commands| collector(&room_id, user_id, commands));
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
				#[cfg(not(test))]
				self.prometheus_user_count.dec(room.template_name.as_str());
				room.disconnect_user(member_and_room_id.member_id)?;
				self.changed_rooms.insert(member_and_room_id.room_id).unwrap();
				Ok(())
			}
		}
	}
}
