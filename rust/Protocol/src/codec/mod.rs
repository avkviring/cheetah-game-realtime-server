use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use chacha20poly1305::aead;
use thiserror::Error;

use crate::codec::cipher::Cipher;
use crate::codec::compress::{packet_compress, packet_decompress};
use crate::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::frame::headers::Headers;
use crate::frame::Frame;
use crate::frame::FRAME_BODY_CAPACITY;

pub mod cipher;
pub mod compress;
pub mod headers;
pub mod variable_int;

#[derive(Error, Debug)]
pub enum FrameDecodeError {
	#[error("Cipher factory error")]
	CipherFactoryError(),
	#[error("DecryptedError {0}")]
	DecryptedError(aead::Error),
	#[error("DecompressError {0}")]
	DecompressError(#[from] snap::Error),
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

impl Frame {
	pub fn decode<'a, F>(data: &'a [u8], chiper_factory: F) -> Result<Frame, FrameDecodeError>
	where
		F: FnOnce(&Headers) -> Option<Cipher<'a>>,
	{
		let mut cursor = Cursor::new(data);
		let connection_id = cursor.read_variable_u64()?;
		let frame_id = cursor.read_variable_u64()?;
		let reliability = cursor.read_u8()? == 1;

		let headers = Headers::decode_headers(&mut cursor)?;
		let header_end = cursor.position();

		let cipher = chiper_factory(&headers).ok_or_else(FrameDecodeError::CipherFactoryError)?;
		let mut frame = Frame {
			connection_id,
			frame_id,
			headers,
			body_size: 0,
			body: [0; FRAME_BODY_CAPACITY],
			reliability,
		};
		frame.body_size = Self::decode_body(cursor, cipher, frame_id, header_end, &mut frame.body)?;
		Ok(frame)
	}

	fn decode_body<'a>(cursor: Cursor<&'a [u8]>, mut cipher: Cipher<'a>, frame_id: u64, header_end: u64, body: &mut [u8; FRAME_BODY_CAPACITY]) -> Result<usize, FrameDecodeError> {
		let data = cursor.into_inner();
		let nonce = frame_id.to_be_bytes();
		let ad = &data[0..header_end as usize];
		let mut vec: heapless::Vec<u8, 4096> = heapless::Vec::new();
		vec.extend_from_slice(&data[header_end as usize..data.len()]).map_err(|_| FrameDecodeError::HeaplessError)?;
		cipher.decrypt(&mut vec, ad, nonce).map_err(FrameDecodeError::DecryptedError)?;
		let decompressed_size = packet_decompress(&vec, body.as_mut_slice())?;
		Ok(decompressed_size)
	}
}

impl Frame {
	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	///
	#[allow(clippy::cast_possible_truncation)]
	pub fn encode(&self, cipher: &mut Cipher<'_>, out: &mut [u8]) -> Result<usize, FrameEncodeError> {
		let mut frame_cursor = Cursor::new(out);
		frame_cursor.write_variable_u64(self.connection_id).map_err(FrameEncodeError::Io)?;
		frame_cursor.write_variable_u64(self.frame_id).map_err(FrameEncodeError::Io)?;
		frame_cursor.write_u8(if self.reliability { 1 } else { 0 }).map_err(FrameEncodeError::Io)?;

		self.headers.encode_headers(&mut frame_cursor).map_err(FrameEncodeError::Io)?;
		let commands_buffer = &self.body[0..self.body_size];

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
	use crate::codec::cipher::Cipher;
	use crate::frame::headers::Header;
	use crate::frame::Frame;
	use crate::reliable::ack::header::AckHeader;

	const PRIVATE_KEY: &[u8] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9, 0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05, 0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f, 0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];

	#[test]
	fn should_encode_decode_frame() {
		let mut frame = Frame::new(100, 55);
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		frame.headers.add(Header::Ack(AckHeader::default()));
		frame.headers.add(Header::Ack(AckHeader::default()));
		frame.set_body(&[1, 2, 3, 4, 6]);

		let mut buffer = [0; 1024];
		let size = frame.encode(&mut cipher, &mut buffer).unwrap();
		let buffer = &buffer[0..size];
		let decoded_frame = Frame::decode(buffer, |_| Some(cipher.clone())).unwrap();

		assert_eq!(decoded_frame, decoded_frame);
	}
}
