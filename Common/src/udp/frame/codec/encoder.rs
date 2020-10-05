use rmps::Serializer;
use serde::Serialize;

use crate::udp::frame::codec::cipher::Cipher;
use crate::udp::frame::codec::compress::packet_compress;
use crate::udp::frame::format::{ApplicationCommand, UdpAdditionalHeader, UdpFrame, UdpFrameHeader};

///
/// Ограничение на размер фрейма
///
impl UdpFrame {
	pub const MAX_FRAME_SIZE: usize = 1024;
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	/// - так как есть ограничение на размер фрейма, то не все команды могут быть преобразованы
	/// - остаток команд возвращается как результат функции
	/// - данные команды также удаляются из исходного фрейма
	///
	pub fn encode(&mut self, cipher: &mut Cipher) -> (Vec<u8>, Vec<ApplicationCommand>) {
		let mut excess_commands = Vec::new();
		
		let mut frame = Vec::new();
		let mut serializer = Serializer::new(&mut frame);
		self.header.serialize(&mut serializer).unwrap();
		self.additional_headers.serialize(&mut serializer).unwrap();
		
		let mut serialized_commands = Vec::new();
		serialized_commands.push(0);// резерв для количества команд
		let mut commands_count = 0;
		self.commands
			.retain(|command| {
				if frame.len() + serialized_commands.len() < UdpFrame::MAX_FRAME_SIZE && commands_count < 255 {
					to_vec(command, &mut serialized_commands);
					commands_count += 1;
					true
				} else {
					excess_commands.push(command.clone());
					false
				}
			});
		
		serialized_commands[0] = commands_count;
		let compressed_commands = packet_compress(&serialized_commands).unwrap();
		let encrypted_commands = cipher.encrypt(&compressed_commands, &frame, self.header.frame_id.to_be_bytes());
		frame.extend_from_slice(&encrypted_commands);
		(frame, excess_commands)
	}
}

fn to_vec<T: Serialize>(item: T, out: &mut Vec<u8>) {
	item.serialize(&mut Serializer::new(out)).unwrap();
}





