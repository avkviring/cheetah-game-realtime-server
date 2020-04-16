use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

use bytebuffer::ByteBuffer;
use log::error;

use crate::relay::network::command::c2s::C2SCommandExecutor;
use crate::relay::room::clients::Client;
use crate::relay::room::room::Room;

/// Декодирование сетевого потока в набор команд
pub struct Decoder {
	client: Rc<Client>,
	commands: Vec<Box<dyn C2SCommandExecutor>>,
	decoders: HashMap<u8, fn(&mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>>>,
}


impl Decoder {
	pub fn new(client: Rc<Client>) -> Decoder {
		Decoder {
			client,
			commands: Default::default(),
			decoders: Default::default(),
		}
	}
	
	pub fn add_decoder(&mut self, command_id: u8, decoder: fn(&mut ByteBuffer) -> Option<Box<dyn C2SCommandExecutor>>) {
		self.decoders.insert(command_id, decoder);
	}
	
	/// декодирование потока
	/// return true - если есть команды для выполнения
	pub fn decode(&mut self, bytes: &mut ByteBuffer) -> bool {
		while bytes.get_wpos() > bytes.get_rpos() {
			let rpos = bytes.get_rpos();
			let command_code = bytes.read_u8().unwrap();
			let decoder = self.decoders.get(&command_code);
			if decoder.is_some() {
				let command: Option<Box<dyn C2SCommandExecutor>> = decoder.unwrap()(bytes);
				if command.is_some() {
					self.commands.push(command.unwrap());
					let data = bytes.read_bytes(bytes.get_wpos() - bytes.get_rpos()).unwrap();
					bytes.clear();
					bytes.write(data.as_slice());
				} else {
					bytes.set_rpos(rpos);
					break;
				}
			} else {
				error!("Wrong command id '{}' in decoder", command_code)
			}
		};
		return self.commands.len() > 0;
	}
	
	/// выполнить входящие команды
	pub fn execute(&mut self, room: &mut Room) {
		for command in self.commands.iter() {
			let rc = self.client.clone();
			command.execute(rc.borrow(), room)
		}
		self.commands.clear()
	}
}