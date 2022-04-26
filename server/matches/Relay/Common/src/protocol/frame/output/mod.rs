use std::io::Cursor;
use std::slice::Iter;

use crate::protocol::codec::commands::context::CommandContext;
use crate::protocol::codec::commands::encoder::encode_command;
use crate::protocol::frame::applications::CommandWithChannel;
use crate::protocol::frame::headers::{Header, Headers};
use crate::protocol::frame::{CommandVec, FrameId};

pub const MAX_ENCODED_COMMANDS_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub struct OutFrame {
	pub frame_id: FrameId,
	pub headers: Headers,
	commands: CommandVec,
	context: CommandContext,
	encoded_size: u64,
	encoded_commands: [u8; MAX_ENCODED_COMMANDS_SIZE * 2],
	full: bool,
	contains_reliability_command: bool,
}
impl OutFrame {
	pub fn new(frame_id: FrameId) -> Self {
		Self {
			frame_id,
			headers: Default::default(),
			commands: Default::default(),
			context: Default::default(),
			encoded_size: 1,
			encoded_commands: [0; MAX_ENCODED_COMMANDS_SIZE * 2],
			full: false,
			contains_reliability_command: false,
		}
	}

	pub fn add_command(&mut self, command: CommandWithChannel) -> Result<(), ()> {
		if self.full {
			return Err(());
		}
		let mut cursor = Cursor::new(self.encoded_commands.as_mut_slice());
		cursor.set_position(self.encoded_size);
		encode_command(&mut self.context, &command, &mut cursor).unwrap();
		if cursor.position() > MAX_ENCODED_COMMANDS_SIZE as u64 {
			self.full = true;
			return Err(());
		} else {
			self.contains_reliability_command = self.contains_reliability_command || command.channel.is_reliable();
			self.encoded_size = cursor.position();
			self.commands.push(command).unwrap();
			self.encoded_commands[0] = self.commands.len() as u8;
		}
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
	pub fn contains_reliability_command(&self) -> bool {
		self.contains_reliability_command
	}

	pub fn get_commands_buffer(&self) -> &[u8] {
		&self.encoded_commands[0..self.encoded_size as usize]
	}
}
