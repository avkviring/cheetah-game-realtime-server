use std::io::Cursor;

use byteorder::ReadBytesExt;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

use crate::udp::protocol::codec::cipher::Cipher;
use crate::udp::protocol::codec::compress::{packet_compress, packet_decompress};
use crate::udp::protocol::frame::{Frame, FrameHeader};
use crate::udp::protocol::frame::applications::{ApplicationCommand, ApplicationCommands};
use crate::udp::protocol::frame::headers::Headers;

#[derive(Debug)]
pub enum UdpFrameDecodeError {
	HeaderDeserializeError,
	AdditionalHeadersDeserializeError,
	ProtocolVersionMismatch,
	DecryptedError,
	DecompressError,
	CommandCountReadError,
	CommandDeserializeError,
}

impl Frame {
	pub fn decode_headers(cursor: &mut Cursor<&[u8]>) -> Result<(FrameHeader, Headers), UdpFrameDecodeError> {
		let mut de = rmp_serde::Deserializer::new(cursor);
		let header: FrameHeader = Deserialize::deserialize(&mut de).map_err(|_| { UdpFrameDecodeError::HeaderDeserializeError })?;
		if header.protocol_version != Frame::PROTOCOL_VERSION {
			Result::Err(UdpFrameDecodeError::ProtocolVersionMismatch)
		} else {
			let additional_headers: Headers = Deserialize::deserialize(&mut de).map_err(|_| { UdpFrameDecodeError::AdditionalHeadersDeserializeError })?;
			Result::Ok((header, additional_headers))
		}
	}
	
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	/// - так как есть ограничение на размер фрейма, то не все команды могут быть преобразованы
	/// - остаток команд возвращается как результат функции
	/// - данные команды также удаляются из исходного фрейма
	///
	/// Метод вызывается после decode_headers (более подробно в тестах)
	///
	pub fn decode_frame(cursor: Cursor<&[u8]>,
						mut cipher: Cipher,
						header: FrameHeader,
						additional_headers: Headers) -> Result<Frame, UdpFrameDecodeError> {
		let header_end = cursor.position();
		let data = cursor.into_inner();
		
		// commands - decrypt
		let nonce = header.frame_id.to_be_bytes() as [u8; 8];
		let ad = &data[0..header_end as usize];
		let msg = &data[header_end as usize..data.len()];
		
		let decrypted = cipher.decrypt(msg, ad, nonce).map_err(|_| { UdpFrameDecodeError::DecryptedError })?;
		// commands - decompress
		let decompressed = packet_decompress(&decrypted).map_err(|_| { UdpFrameDecodeError::DecompressError })?;
		
		let mut cursor = Cursor::new(decompressed);
		let unreliability = Frame::decode_commands(&mut cursor)?;
		let reliability = Frame::decode_commands(&mut cursor)?;
		
		Result::Ok(Frame {
			header,
			headers: additional_headers,
			commands: ApplicationCommands
			{
				reliability,
				unreliability,
			},
		})
	}
	
	fn decode_commands(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<ApplicationCommand>, UdpFrameDecodeError> {
		let mut commands = Vec::new();
		let commands_count = cursor.read_u8().map_err(|_| { UdpFrameDecodeError::CommandCountReadError })?;
		let mut deserializer = rmp_serde::Deserializer::new(cursor);
		for _ in 0..commands_count {
			let command = Deserialize::deserialize(&mut deserializer).map_err(|_| { UdpFrameDecodeError::CommandDeserializeError })?;
			commands.push(command);
		}
		Result::Ok(commands)
	}
	
	pub const MAX_FRAME_SIZE: usize = 1024;
	
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	/// - так как есть ограничение на размер фрейма, то не все команды могут быть преобразованы
	/// - остаток команд возвращается как результат функции
	/// - данные команды также удаляются из исходного фрейма
	///
	pub fn encode(&mut self, cipher: &mut Cipher) -> (Vec<u8>, ApplicationCommands) {
		let mut frame = Vec::new();
		let mut serializer = Serializer::new(&mut frame);
		self.header.serialize(&mut serializer).unwrap();
		self.headers.serialize(&mut serializer).unwrap();
		
		let mut serialized_commands = Vec::new();
		let unreliability_remaining =
			Frame::serialized_commands(
				&mut self.commands.unreliability,
				frame.len(),
				&mut serialized_commands);
		
		let reliability_remaining =
			Frame::serialized_commands(
				&mut self.commands.reliability,
				frame.len(),
				&mut serialized_commands);
		
		let compressed_commands = packet_compress(&serialized_commands).unwrap();
		let encrypted_commands = cipher.encrypt(&compressed_commands, &frame, self.header.frame_id.to_be_bytes());
		frame.extend_from_slice(&encrypted_commands);
		(frame, ApplicationCommands { reliability: reliability_remaining, unreliability: unreliability_remaining })
	}
	
	fn serialized_commands(commands: &mut Vec<ApplicationCommand>, frame_length: usize, out: &mut Vec<u8>) -> Vec<ApplicationCommand> {
		out.push(0);
		let position = out.len() - 1;
		let mut commands_count = 0;
		let mut remaining_commands = Vec::new();
		commands.retain(|command| {
			if frame_length + out.len() < Frame::MAX_FRAME_SIZE && commands_count < 255 {
				to_vec(command, out);
				commands_count += 1;
				true
			} else {
				remaining_commands.push(command.clone());
				false
			}
		});
		out[position] = commands_count;
		remaining_commands
	}
}

fn to_vec<T: Serialize>(item: T, out: &mut Vec<u8>) {
	item.serialize(&mut Serializer::new(out)).unwrap();
}


#[cfg(test)]
pub mod tests {
	use std::io::Cursor;
	
	use crate::udp::protocol::codec::cipher::Cipher;
	use crate::udp::protocol::frame::applications::ApplicationCommand;
	use crate::udp::protocol::frame::Frame;
	use crate::udp::protocol::frame::headers::Header;
	use crate::udp::protocol::reliable::ack::header::AckFrameHeader;
	
	const PRIVATE_KEY: &[u8; 32] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9,
		0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05,
		0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f,
		0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];
	
	#[test]
	fn should_encode_decode_frame() {
		let mut frame = Frame::new(0);
		let mut cipher = Cipher::new(PRIVATE_KEY);
		frame.headers.add(Header::AckFrame(AckFrameHeader::new(10)));
		frame.headers.add(Header::AckFrame(AckFrameHeader::new(15)));
		frame.commands.reliability.push(ApplicationCommand::Ping("test".to_string()));
		let (data, _) = frame.encode(&mut cipher);
		
		let mut cursor = Cursor::new(data.as_slice());
		let (header, additional_header) = Frame::decode_headers(&mut cursor).unwrap();
		let decoded_frame = Frame::decode_frame(cursor, cipher.clone(), header, additional_header).unwrap();
		
		assert_eq!(frame, decoded_frame);
	}
	
	
	#[test]
	fn should_limit_buffer_size() {
		let mut frame = Frame::new(0);
		let mut cipher = Cipher::new(PRIVATE_KEY);
		const COMMAND_COUNT: usize = 400;
		for _ in 0..COMMAND_COUNT {
			frame.commands.reliability.push(ApplicationCommand::Ping("1234567890".to_string()));
		}
		let (data, remaining_commands) = frame.encode(&mut cipher);
		
		assert!(data.len() <= Frame::MAX_FRAME_SIZE);
		assert_eq!(remaining_commands.reliability.len() + frame.commands.reliability.len(), COMMAND_COUNT);
		
		let mut cursor = Cursor::new(data.as_slice());
		let (header, additional_header) = Frame::decode_headers(&mut cursor).unwrap();
		let decoded_frame = Frame::decode_frame(cursor, cipher.clone(), header, additional_header).unwrap();
		
		assert_eq!(frame, decoded_frame);
	}
}

