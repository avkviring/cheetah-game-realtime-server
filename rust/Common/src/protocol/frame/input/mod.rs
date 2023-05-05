use std::slice::Iter;

use crate::protocol::frame::applications::CommandWithReliabilityGuarantees;
use crate::protocol::frame::headers::{Header, Headers};
use crate::protocol::frame::{ConnectionId, FrameId};

#[derive(Debug, PartialEq, Clone)]
pub struct InFrame {
	pub connection_id: ConnectionId,
	pub frame_id: FrameId,
	pub headers: Headers,
	commands: Vec<CommandWithReliabilityGuarantees>,
	contains_reliability_command: bool,
}
impl InFrame {
	#[must_use]
	pub fn new(connection_id: ConnectionId, frame_id: FrameId, headers: Headers, commands: Vec<CommandWithReliabilityGuarantees>) -> Self {
		let contains_reliability_command = commands.iter().any(|f| f.reliability_guarantees.is_reliable());
		Self {
			connection_id,
			frame_id,
			headers,
			commands,
			contains_reliability_command,
		}
	}

	pub fn get_commands(&self) -> Iter<'_, CommandWithReliabilityGuarantees> {
		self.commands.iter()
	}

	///
	///  Получить оригинальный `frame_id`
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	///
	#[must_use]
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_retransmit) {
			None => self.frame_id,
			Some(value) => value.original_frame_id,
		}
	}

	///
	/// Фрейм с надежной доставкой?
	///
	#[must_use]
	pub fn contains_reliability_command(&self) -> bool {
		self.contains_reliability_command
	}
}
