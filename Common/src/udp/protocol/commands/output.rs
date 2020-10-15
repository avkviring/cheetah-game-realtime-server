use std::time::Instant;

use crate::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommands};
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::FrameBuilder;

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Default)]
pub struct OutCommandsCollector {
	commands: ApplicationCommands
}

impl OutCommandsCollector {
	pub fn add_unsent_commands(&mut self, commands: ApplicationCommands) {
		self.commands.add(&commands);
	}
	pub fn add_reliability_command(&mut self, command: ApplicationCommand) {
		self.commands.reliability.push(command);
	}
	pub fn add_unreliability_command(&mut self, command: ApplicationCommand) {
		self.commands.unreliability.push(command);
	}
}


impl FrameBuilder for OutCommandsCollector {
	fn contains_self_data(&self, _: &Instant) -> bool {
		self.commands.reliability.len() + self.commands.unreliability.len() > 0
	}
	
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		frame.commands.add(&self.commands);
		self.commands.clear();
	}
}