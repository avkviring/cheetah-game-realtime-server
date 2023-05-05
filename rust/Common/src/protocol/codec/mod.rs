use std::io::{Cursor, Write};

use chacha20poly1305::aead;
use thiserror::Error;

use crate::protocol::codec::cipher::Cipher;
use crate::protocol::codec::commands::decoder::{decode_commands, CommandsDecoderError};
use crate::protocol::codec::compress::{packet_compress, packet_decompress};
use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::applications::CommandWithReliabilityGuarantees;
use crate::protocol::frame::headers::Headers;
use crate::protocol::frame::input::InFrame;
use crate::protocol::frame::output::OutFrame;
use crate::protocol::frame::{ConnectionId, FrameId, MAX_FRAME_SIZE};

pub mod channel;
pub mod cipher;
pub mod commands;
pub mod compress;
pub mod headers;
pub mod variable_int;

#[derive(Error, Debug)]
pub enum FrameDecodeError {
	#[error("DecryptedError {0}")]
	DecryptedError(aead::Error),
	#[error("DecompressError {0}")]
	DecompressError(#[from] snap::Error),
	#[error("Decode commands error {0}")]
	CommandsDecode(#[from] CommandsDecoderError),
	#[error("Io error {0}")]
	Io(#[from] std::io::Error),
	#[error("HeaplessError")]
	HeaplessError,
}

#[derive(Error, Debug)]
pub enum FrameEncodeError {
	#[error("EncryptedError {0}")]
	EncryptedError(aead::Error),
	#[error("CompressError {0}")]
	CompressError(#[from] snap::Error),
	#[error("Io error {0}")]
	Io(#[from] std::io::Error),
}

impl InFrame {
	pub fn decode_meta(cursor: &mut Cursor<&[u8]>) -> Result<(ConnectionId, FrameId, Headers), FrameDecodeError> {
		let connection_id = cursor.read_variable_u64()?;
		let frame_id = cursor.read_variable_u64()?;
		let headers = Headers::decode_headers(cursor)?;
		Ok((connection_id, frame_id, headers))
	}

	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	/// - так как есть ограничение на размер фрейма, то не все команды могут быть преобразованы
	/// - остаток команд возвращается как результат функции
	/// - данные команды также удаляются из исходного фрейма
	///
	/// Метод вызывается после `decode_headers` (более подробно в тестах)
	///
	#[allow(clippy::cast_possible_truncation)]
	#[allow(clippy::map_err_ignore)]
	pub fn decode_commands(c2s_commands: bool, frame_id: FrameId, cursor: Cursor<&[u8]>, mut cipher: Cipher<'_>) -> Result<Vec<CommandWithReliabilityGuarantees>, FrameDecodeError> {
		let header_end = cursor.position();
		let data = cursor.into_inner();

		// commands - decrypt
		let nonce = frame_id.to_be_bytes();
		let ad = &data[0..header_end as usize];

		let mut vec: heapless::Vec<u8, 4096> = heapless::Vec::new();
		vec.extend_from_slice(&data[header_end as usize..data.len()]).map_err(|_| FrameDecodeError::HeaplessError)?;

		cipher.decrypt(&mut vec, ad, nonce).map_err(FrameDecodeError::DecryptedError)?;

		// commands - decompress
		let mut decompressed_buffer = [0; MAX_FRAME_SIZE];
		let decompressed_size = packet_decompress(&vec, &mut decompressed_buffer)?;
		let decompressed_buffer = &decompressed_buffer[0..decompressed_size];

		let mut cursor = Cursor::new(decompressed_buffer);

		let mut commands = Default::default();
		decode_commands(c2s_commands, &mut cursor, &mut commands)?;
		Ok(commands)
	}
}

impl OutFrame {
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	///
	#[allow(clippy::cast_possible_truncation)]
	pub fn encode(&self, cipher: &mut Cipher<'_>, out: &mut [u8]) -> Result<usize, FrameEncodeError> {
		let mut frame_cursor = Cursor::new(out);
		frame_cursor.write_variable_u64(self.connection_id).map_err(FrameEncodeError::Io)?;
		frame_cursor.write_variable_u64(self.frame_id).map_err(FrameEncodeError::Io)?;

		self.headers.encode_headers(&mut frame_cursor).map_err(FrameEncodeError::Io)?;
		let commands_buffer = self.get_commands_buffer();

		let mut vec: heapless::Vec<u8, 4096> = heapless::Vec::new();
		unsafe {
			vec.set_len(4096);
		}
		let compressed_size = packet_compress(commands_buffer, &mut vec)?;
		unsafe {
			vec.set_len(compressed_size);
		}

		let frame_position = frame_cursor.position() as usize;
		cipher
			.encrypt(&mut vec, &frame_cursor.get_ref()[0..frame_position], self.frame_id.to_be_bytes())
			.map_err(FrameEncodeError::EncryptedError)?;

		frame_cursor.write_all(&vec)?;

		Ok(frame_cursor.position() as usize)
	}
}

#[cfg(test)]
pub mod tests {
	use std::io::Cursor;

	use crate::commands::c2s::C2SCommand;
	use crate::commands::types::long::SetLongCommand;
	use crate::protocol::codec::cipher::Cipher;
	use crate::protocol::frame::applications::{BothDirectionCommand, CommandWithReliabilityGuarantees};
	use crate::protocol::frame::channel::ReliabilityGuaranteesChannel;
	use crate::protocol::frame::headers::Header;
	use crate::protocol::frame::input::InFrame;
	use crate::protocol::frame::output::OutFrame;
	use crate::protocol::reliable::ack::header::AckHeader;
	use crate::room::object::GameObjectId;
	use crate::room::owner::GameObjectOwner;

	const PRIVATE_KEY: &[u8] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9, 0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05, 0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f, 0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];

	#[test]
	fn should_encode_decode_frame() {
		let mut frame = OutFrame::new(100, 55);
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		frame.headers.add(Header::Ack(AckHeader::default()));
		frame.headers.add(Header::Ack(AckHeader::default()));
		frame.add_command(CommandWithReliabilityGuarantees {
			reliability_guarantees: ReliabilityGuaranteesChannel::ReliableUnordered,
			commands: BothDirectionCommand::C2S(C2SCommand::SetLong(SetLongCommand {
				object_id: GameObjectId::new(100, GameObjectOwner::Member(200)),
				field_id: 78,
				value: 155,
			})),
		});
		let mut buffer = [0; 1024];
		let size = frame.encode(&mut cipher, &mut buffer).unwrap();
		let buffer = &buffer[0..size];

		let mut cursor = Cursor::new(buffer);
		let (connection_id, frame_id, headers) = InFrame::decode_meta(&mut cursor).unwrap();
		let commands = InFrame::decode_commands(true, frame_id, cursor, cipher.clone()).unwrap();
		let decoded_frame = InFrame::new(connection_id, frame_id, headers, commands);

		assert_eq!(frame.frame_id, decoded_frame.frame_id);
		assert_eq!(frame.headers, decoded_frame.headers);
		assert_eq!(frame.get_commands().as_slice(), decoded_frame.get_commands().as_slice());
	}
}
