use std::io::Cursor;
use std::slice::Iter;

use crate::commands::c2s::C2SCommand::DeleteField;
use crate::protocol::codec::commands::encoder::encode_commands;
use crate::protocol::frame::applications::CommandWithChannel;
use crate::protocol::frame::headers::{Header, Headers};
use crate::protocol::frame::{CommandVec, FrameId};

#[derive(Debug, PartialEq, Clone)]
pub struct OutFrame {
	pub frame_id: FrameId,
	pub headers: Headers,
	commands: CommandVec,
}
impl OutFrame {
	pub fn new(frame_id: FrameId) -> Self {
		Self {
			frame_id,
			headers: Default::default(),
			commands: Default::default(),
		}
	}

	pub fn add_command(&mut self, command: CommandWithChannel) -> Result<(), ()> {
		self.commands.push(command);
		Ok(())
	}

	pub fn get_commands(&self) -> Iter<'_, CommandWithChannel> {
		self.commands.iter()
	}

	///
	///  Получить оригинальный frame_id
	/// - для повторно отосланных фреймов - id изначального фрейма
	/// - для всех остальных id фрейма
	///
	pub fn get_original_frame_id(&self) -> FrameId {
		match self.headers.first(Header::predicate_retransmit) {
			None => self.frame_id,
			Some(value) => value.original_frame_id,
		}
	}
	pub fn is_reliability(&self) -> bool {
		self.commands.iter().any(|f| f.channel.is_reliable())
	}

	pub fn get_commands_buffer(&self, out: &mut Cursor<&mut [u8]>) {
		encode_commands(&self.commands, out).unwrap();
	}
}
