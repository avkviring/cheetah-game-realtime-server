use std::time::Instant;

use crate::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommands};
use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::FrameReceivedListener;

///
/// Коллектор входящих команд
///
#[derive(Default)]
pub struct InCommandsCollector {
	commands: ApplicationCommands
}

impl InCommandsCollector {
	pub fn get_and_remove_commands(&mut self) -> Vec<ApplicationCommand> {
		let mut result = Vec::new();
		result.extend_from_slice(&self.commands.reliability);
		result.extend_from_slice(&self.commands.unreliability);
		self.commands.reliability.clear();
		self.commands.unreliability.clear();
		result
	}
}

impl FrameReceivedListener for InCommandsCollector {
	fn on_frame_received(&mut self, frame: &Frame, _: &Instant) {
		self.commands.add(&frame.commands);
	}
}