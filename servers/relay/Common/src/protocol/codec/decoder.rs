use std::io::{Cursor, Write};

use rmp_serde::Serializer;
use serde::Serialize;

use crate::protocol::codec::cipher::Cipher;
use crate::protocol::codec::compress::{packet_compress, packet_decompress};
use crate::protocol::codec::serializer::{deserialize, serialize};
use crate::protocol::frame::headers::Headers;
use crate::protocol::frame::{Frame, FrameHeader};

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
		let header: FrameHeader = deserialize(cursor).map_err(|_| UdpFrameDecodeError::HeaderDeserializeError)?;
		if header.protocol_version != Frame::PROTOCOL_VERSION {
			Result::Err(UdpFrameDecodeError::ProtocolVersionMismatch)
		} else {
			let additional_headers: Headers = deserialize(cursor).map_err(|_| UdpFrameDecodeError::AdditionalHeadersDeserializeError)?;
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
	pub fn decode_frame(cursor: Cursor<&[u8]>, mut cipher: Cipher, header: FrameHeader, headers: Headers) -> Result<Frame, UdpFrameDecodeError> {
		let header_end = cursor.position();
		let data = cursor.into_inner();

		// commands - decrypt
		let nonce = header.frame_id.to_be_bytes() as [u8; 8];
		let ad = &data[0..header_end as usize];

		let mut vec: heapless::Vec<u8, heapless::consts::U1024> = heapless::Vec::new();
		vec.extend_from_slice(&data[header_end as usize..data.len()]).unwrap();

		cipher.decrypt(&mut vec, ad, nonce).map_err(|_| UdpFrameDecodeError::DecryptedError)?;

		// commands - decompress
		let mut decompressed_buffer = [0; Frame::MAX_FRAME_SIZE];
		let decompressed_size = packet_decompress(&vec, &mut decompressed_buffer).map_err(|_| UdpFrameDecodeError::DecompressError)?;
		let decompressed_buffer = &decompressed_buffer[0..decompressed_size];

		let commands = deserialize(&mut Cursor::new(decompressed_buffer)).map_err(|_| UdpFrameDecodeError::CommandDeserializeError)?;

		Result::Ok(Frame { header, headers, commands })
	}

	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	///
	pub fn encode(&self, cipher: &mut Cipher, out: &mut [u8]) -> usize {
		let mut frame_cursor = Cursor::new(out);
		let mut serializer = Serializer::new(&mut frame_cursor);
		self.header.serialize(&mut serializer).unwrap();
		self.headers.serialize(&mut serializer).unwrap();
		drop(serializer);

		let mut commands_buffer = [0 as u8; Frame::MAX_FRAME_SIZE];
		let mut commands_cursor = Cursor::new(&mut commands_buffer[..]);
		serialize(&self.commands, &mut commands_cursor);

		let mut vec: heapless::Vec<u8, heapless::consts::U1024> = heapless::Vec::new();
		unsafe {
			vec.set_len(Frame::MAX_FRAME_SIZE);
		}

		let commands_position = commands_cursor.position() as usize;
		let compressed_size = packet_compress(&commands_buffer[0..commands_position], &mut vec).unwrap();
		unsafe {
			vec.set_len(compressed_size);
		}

		let frame_position = frame_cursor.position() as usize;
		cipher
			.encrypt(&mut vec, &frame_cursor.get_ref()[0..frame_position], self.header.frame_id.to_be_bytes())
			.unwrap();

		frame_cursor.write_all(&vec).unwrap();

		frame_cursor.position() as usize
	}
}

#[cfg(test)]
pub mod tests {
	use std::io::Cursor;

	use crate::protocol::codec::cipher::Cipher;
	use crate::protocol::frame::applications::{ApplicationCommand, ApplicationCommandChannel, ApplicationCommandDescription};
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::Frame;
	use crate::protocol::reliable::ack::header::AckFrameHeader;

	const PRIVATE_KEY: &[u8; 32] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9, 0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05, 0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad,
		0x0f, 0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];

	#[test]
	fn should_encode_decode_frame() {
		let mut frame = Frame::new(0);
		let mut cipher = Cipher::new(PRIVATE_KEY);
		frame.headers.add(Header::AckFrame(AckFrameHeader::new(10)));
		frame.headers.add(Header::AckFrame(AckFrameHeader::new(15)));
		frame.commands.reliable.push_back(ApplicationCommandDescription {
			channel: ApplicationCommandChannel::ReliableUnordered,
			command: ApplicationCommand::TestSimple("test".to_string()),
		});
		let mut buffer = [0; 1024];
		let size = frame.encode(&mut cipher, &mut buffer);
		let buffer = &buffer[0..size];

		let mut cursor = Cursor::new(buffer);
		let (header, additional_header) = Frame::decode_headers(&mut cursor).unwrap();
		let decoded_frame = Frame::decode_frame(cursor, cipher.clone(), header, additional_header).unwrap();

		assert_eq!(frame, decoded_frame);
	}
}
