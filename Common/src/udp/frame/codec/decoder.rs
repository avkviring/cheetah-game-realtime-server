use std::io::Cursor;

use byteorder::ReadBytesExt;
use serde::de::{Deserialize, Deserializer};

use crate::udp::frame::codec::cipher::Cipher;
use crate::udp::frame::codec::compress::packet_decompress;
use crate::udp::frame::format::{UdpAdditionalHeader, UdpFrame, UdpFrameHeader};

pub enum UdpFrameDecodeError {
	HeaderDeserializeError,
	AdditionalHeadersDeserializeError,
	CipherNotFound,
	DecryptedError,
	DecompressError,
	CommandCountReadError,
	CommandDeserializeError,
}

impl UdpFrame {
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	/// - так как есть ограничение на размер фрейма, то не все команды могут быть преобразованы
	/// - остаток команд возвращается как результат функции
	/// - данные команды также удаляются из исходного фрейма
	///
	pub fn decode<F>(cipher_finder: F, data: &[u8]) -> Result<UdpFrame, UdpFrameDecodeError> where F: FnOnce(&Vec<UdpAdditionalHeader>) -> Result<Cipher, ()> {
		// header
		let mut header_read_cursor = Cursor::new(data);
		let mut de = rmps::Deserializer::new(&mut header_read_cursor);
		let header: UdpFrameHeader = Deserialize::deserialize(&mut de).map_err(|_| { UdpFrameDecodeError::HeaderDeserializeError })?;
		let additional_headers: Vec<UdpAdditionalHeader> = Deserialize::deserialize(&mut de).map_err(|_| { UdpFrameDecodeError::AdditionalHeadersDeserializeError })?;
		
		// commands - decrypt
		let nonce = header.frame_id.to_be_bytes() as [u8; 8];
		let header_end = header_read_cursor.position();
		let ad = &data[0..header_end as usize];
		let msg = &data[header_end as usize..data.len()];
		
		let mut cipher = cipher_finder(&additional_headers).map_err(|_| { UdpFrameDecodeError::CipherNotFound })?;
		let decrypted = cipher.decrypt(msg, ad, nonce).map_err(|_| { UdpFrameDecodeError::DecryptedError })?;
		// commands - decompress
		let decompressed = packet_decompress(&decrypted).map_err(|e| { UdpFrameDecodeError::DecompressError })?;
		
		
		let mut commands = Vec::new();
		let mut cursor = Cursor::new(decompressed);
		// commands - decode
		let command_count = cursor.read_u8().map_err(|e| { UdpFrameDecodeError::CommandCountReadError })?;
		let mut deserializer = rmps::Deserializer::new(cursor);
		
		for _ in 0..command_count {
			let command = Deserialize::deserialize(&mut deserializer).map_err(|e| { UdpFrameDecodeError::CommandDeserializeError })?;
			commands.push(command);
		}
		
		Result::Ok(UdpFrame {
			header,
			additional_headers,
			commands,
		})
	}
}


