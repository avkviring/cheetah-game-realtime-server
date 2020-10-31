use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use heapless::consts::*;
use heapless::Vec as HeaplessVec;
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
		
		let mut vec: HeaplessVec<u8, U2048> = HeaplessVec::new();
		vec.extend_from_slice(&data[header_end as usize..data.len()]);
		
		cipher.decrypt(&mut vec, ad, nonce).map_err(|_| { UdpFrameDecodeError::DecryptedError })?;
		
		// commands - decompress
		let mut decompressed_buffer = [0; 2048];
		let decompressed_size = packet_decompress(&mut vec, &mut decompressed_buffer).map_err(|_| { UdpFrameDecodeError::DecompressError })?;
		let decompressed_buffer = &decompressed_buffer[0..decompressed_size];
		
		
		let mut cursor = Cursor::new(decompressed_buffer);
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
	
	fn decode_commands(cursor: &mut Cursor<&[u8]>) -> Result<Vec<ApplicationCommand>, UdpFrameDecodeError> {
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
	pub fn encode(&mut self, cipher: &mut Cipher, out: &mut [u8]) -> (ApplicationCommands, usize) {
		let mut frame_cursor = Cursor::new(out);
		let mut serializer = Serializer::new(&mut frame_cursor);
		self.header.serialize(&mut serializer);
		self.headers.serialize(&mut serializer).unwrap();
		drop(serializer);
		
		
		let mut commands_buffer = [0 as u8; Frame::MAX_FRAME_SIZE * 2];
		let mut commands_cursor = Cursor::new(&mut commands_buffer[..]);
		let unreliability_remaining =
			Frame::serialized_commands(
				&mut self.commands.unreliability,
				frame_cursor.position(),
				&mut commands_cursor);
		
		let reliability_remaining =
			Frame::serialized_commands(
				&mut self.commands.reliability,
				frame_cursor.position(),
				&mut commands_cursor);
		
		let mut vec: HeaplessVec<u8, U2048> = HeaplessVec::new();
		unsafe { vec.set_len(2048); }
		
		let commands_position = commands_cursor.position() as usize;
		let compressed_size = packet_compress(&commands_buffer[0..commands_position], &mut vec).unwrap();
		unsafe { vec.set_len(compressed_size); }
		
		let frame_position = frame_cursor.position() as usize;
		cipher.encrypt(
			&mut vec,
			&frame_cursor.get_ref()[0..frame_position],
			self.header.frame_id.to_be_bytes())
			.unwrap();
		
		
		frame_cursor.write_all(&vec).unwrap();
		
		(ApplicationCommands { reliability: reliability_remaining, unreliability: unreliability_remaining }, frame_cursor.position() as usize)
	}
	
	fn serialized_commands(commands: &mut Vec<ApplicationCommand>, frame_length: u64, out: &mut Cursor<&mut [u8]>) -> Vec<ApplicationCommand> {
		let head_position = out.position();
		out.write_u8(0);
		let mut commands_count = 0;
		let mut remaining_commands = Vec::new();
		commands.retain(|command| {
			if frame_length + out.position() < Frame::MAX_FRAME_SIZE as u64 && commands_count < 255 {
				to_vec(command, out);
				commands_count += 1;
				true
			} else {
				remaining_commands.push(command.clone());
				false
			}
		});
		let position = out.position();
		out.set_position(head_position);
		out.write_u8(commands_count);
		out.set_position(position);
		
		remaining_commands
	}
}

fn to_vec<T: Serialize>(item: T, out: &mut Cursor<&mut [u8]>) {
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
		let mut buffer = [0; 1024];
		let (_, size) = frame.encode(&mut cipher, &mut buffer);
		let buffer = &buffer[0..size];
		
		let mut cursor = Cursor::new(buffer);
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
		let mut buffer = [0; 1024];
		let (remaining_commands, size) = frame.encode(&mut cipher, &mut buffer);
		let buffer = &buffer[0..size];
		
		assert!(buffer.len() <= Frame::MAX_FRAME_SIZE);
		assert_eq!(remaining_commands.reliability.len() + frame.commands.reliability.len(), COMMAND_COUNT);
		
		let mut cursor = Cursor::new(buffer);
		let (header, additional_header) = Frame::decode_headers(&mut cursor).unwrap();
		let decoded_frame = Frame::decode_frame(cursor, cipher.clone(), header, additional_header).unwrap();
		
		assert_eq!(frame, decoded_frame);
	}
}

