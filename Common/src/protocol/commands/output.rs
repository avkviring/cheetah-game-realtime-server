use std::time::Instant;

use crate::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription, ApplicationCommands};
use crate::protocol::frame::Frame;
use crate::protocol::FrameBuilder;

///
/// Коллектор команд для отправки
///
/// - удаление дубликатов команд
/// - sequence команды
///
#[derive(Default, Debug)]
pub struct OutCommandsCollector {
    commands: ApplicationCommands
}

impl OutCommandsCollector {
    pub fn add_unsent_commands(&mut self, commands: ApplicationCommands) {
        self.commands.add_first(&commands);
    }

    pub fn add_command(&mut self, channel: ApplicationCommandChannel, command: ApplicationCommand) {
        let commands = match channel {
            ApplicationCommandChannel::ReliableUnordered
            | ApplicationCommandChannel::ReliableOrderedByObject
            | ApplicationCommandChannel::ReliableOrderedByGroup(_)
            | ApplicationCommandChannel::ReliableSequenceByObject(_)
            | ApplicationCommandChannel::ReliableSequenceByGroup(_, _)
            => {
                &mut self.commands.reliable
            }

            ApplicationCommandChannel::UnreliableUnordered
            | ApplicationCommandChannel::UnreliableOrderedByObject
            | ApplicationCommandChannel::UnreliableOrderedByGroup(_) => {
                &mut self.commands.unreliable
            }
        };
        commands.push(ApplicationCommandDescription { channel, command });
    }
}


impl FrameBuilder for OutCommandsCollector {
    fn contains_self_data(&self, _: &Instant) -> bool {
        self.commands.reliable.len() + self.commands.unreliable.len() > 0
    }

    fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
        frame.commands.add_first(&self.commands);
        self.commands.clear();
    }
}