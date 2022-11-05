use std::slice::Iter;

use crate::protocol::frame::applications::CommandWithChannel;
use crate::protocol::frame::headers::{Header, Headers};
use crate::protocol::frame::FrameId;

#[derive(Debug, PartialEq, Clone)]
pub struct InFrame {
	pub frame_id: FrameId,
	pub headers: Headers,
	commands: Vec<CommandWithChannel>,
	contains_reliability_command: bool,
}
impl InFrame {
	#[must_use]
	pub fn new(frame_id: FrameId, headers: Headers, commands: Vec<CommandWithChannel>) -> Self {
		let contains_reliability_command = commands.iter().any(|f| f.channel.is_reliable());
		Self {
			frame_id,
			headers,
			commands,
			contains_reliability_command,
		}
	}

	pub fn get_commands(&self) -> Iter<'_, CommandWithChannel> {
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
