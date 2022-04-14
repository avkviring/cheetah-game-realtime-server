use std::collections::HashMap;

use fnv::FnvBuildHasher;
use prometheus::{IntCounter, IntGauge};

use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_matches_relay_common::protocol::others::user_id::MemberAndRoomId;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId};
use cheetah_microservice::prometheus::measurer::{LabelFactoryFactory, MeasurerByLabel};

use crate::room::command::ServerCommandError;
use crate::room::template::config::{MemberTemplate, RoomTemplate};
use crate::room::Room;

pub struct Rooms {
	pub room_by_id: HashMap<RoomId, Room, FnvBuildHasher>,
	room_id_generator: RoomId,
	changed_rooms: heapless::FnvIndexSet<RoomId, 10_000>,
	measure_room_count: MeasurerByLabel<String, IntGauge>,
	measure_user_count: MeasurerByLabel<String, IntGauge>,
	measure_income_command_count: MeasurerByLabel<(Option<FieldType>, Option<FieldId>, heapless::String<50>), IntCounter>,
	measure_outcome_command_count: MeasurerByLabel<(Option<FieldType>, Option<FieldId>, heapless::String<50>), IntCounter>,
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
			measure_room_count: MeasurerByLabel::new(
				"room_count",
				"Room by template",
				prometheus::default_registry().clone(),
				Box::new(|template| vec![("template", template.clone())]),
			),
			measure_user_count: MeasurerByLabel::new(
				"user_count",
				"User count",
				prometheus::default_registry().clone(),
				Box::new(|template| vec![("template", template.clone())]),
			),
			measure_income_command_count: MeasurerByLabel::new(
				"income_command_counter",
				"Income command counter",
				prometheus::default_registry().clone(),
				Self::measurer_label_factory(),
			),
			measure_outcome_command_count: MeasurerByLabel::new(
				"outcome_command_counter",
				"Outcome command counter",
				prometheus::default_registry().clone(),
				Self::measurer_label_factory(),
			),
		}
	}
}

impl Rooms {
	pub fn create_room(&mut self, template: RoomTemplate) -> RoomId {
		self.room_id_generator += 1;
		self.measure_room_count.measurer(&template.name).inc();
		let room_id = self.room_id_generator;
		let room = Room::new(room_id, template);
		self.room_by_id.insert(room_id, room);
		room_id
	}

	pub fn register_user(&mut self, room_id: RoomId, member_template: MemberTemplate) -> Result<RoomMemberId, RegisterUserError> {
		match self.room_by_id.get_mut(&room_id) {
			None => Result::Err(RegisterUserError::RoomNotFound),
			Some(room) => {
				self.measure_user_count.measurer(&room.template_name).inc();
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
					let template = heapless::String::<50>::from(room.template_name.as_str());
					room.collect_out_commands(|user_id, commands| {
						collector(&room_id, user_id, commands);
						commands.iter().for_each(|c| {
							if let BothDirectionCommand::S2CWithCreator(ref c) = c.command {
								let c = &c.command;
								let key = (c.get_field_type(), c.get_field_id(), template.clone());
								self.measure_outcome_command_count.measurer(&key).inc();
							}
						});
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
				Self::measure_income_commands(&mut self.measure_income_command_count, commands, template);
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
				self.measure_user_count.measurer(&room.template_name);
				room.disconnect_user(member_and_room_id.member_id)?;
				self.changed_rooms.insert(member_and_room_id.room_id).unwrap();
				Ok(())
			}
		}
	}

	fn measure_income_commands(
		measurers: &mut MeasurerByLabel<(Option<FieldType>, Option<FieldId>, heapless::String<50>), IntCounter>,
		commands: &[CommandWithChannel],
		template: heapless::String<50>,
	) {
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::C2S(ref c) = c.both_direction_command {
				let key = (c.get_field_type(), c.get_field_id(), template.clone());
				measurers.measurer(&key).inc()
			}
		});
	}
	fn measurer_label_factory() -> Box<LabelFactoryFactory<(Option<FieldType>, Option<FieldId>, heapless::String<50>)>> {
		Box::new(|(t, id, template)| {
			vec![
				(
					"field_type",
					t.map(|f| Into::<&str>::into(f).into())
						.unwrap_or_else(|| "unknown".to_string()),
				),
				(
					"field_id",
					id.map(|f| format!("{}", f)).unwrap_or_else(|| "unknown".to_string()),
				),
				("template", template.to_string()),
			]
		})
	}
}
