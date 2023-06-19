use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};
use chacha20poly1305::aead;
use thiserror::Error;

use crate::codec::cipher::Cipher;
use crate::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::frame::headers::Headers;
use crate::frame::segment::Segment;
use crate::frame::Frame;

pub mod cipher;
pub mod compress;
pub mod headers;
pub mod segment;
pub mod variable_int;

#[derive(Error, Debug)]
pub enum DecodeError {
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
pub enum EncodeError {
	#[error("EncryptedError {0}")]
	EncryptedError(aead::Error),
	#[error("CompressError {0}")]
	CompressError(#[from] snap::Error),
	#[error("Io error {0}")]
	Io(#[from] std::io::Error),
}

impl Frame {
	pub fn decode<'a, F>(data: &'a [u8], chiper_factory: F) -> Result<Frame, DecodeError>
	where
		F: FnOnce(&Headers) -> Option<Cipher<'a>>,
	{
		let mut cursor = Cursor::new(data);
		let connection_id = cursor.read_variable_u64()?;
		let frame_id = cursor.read_variable_u64()?;
		let reliability = cursor.read_u8()? == 1;
		let headers = Headers::decode_headers(&mut cursor)?;

		let cipher = chiper_factory(&headers).ok_or_else(DecodeError::CipherFactoryError)?;
		let segment = Segment::decode(cursor, cipher, frame_id)?;
		let frame = Frame {
			connection_id,
			frame_id,
			headers,
			reliability,
			segment,
		};

		Ok(frame)
	}

	///
	/// Преобразуем Frame в набор байт для отправки через сеть
	///
	#[allow(clippy::cast_possible_truncation)]
	pub fn encode(&self, cipher: &mut Cipher<'_>, out: &mut [u8]) -> Result<usize, EncodeError> {
		let mut frame_cursor = Cursor::new(out);
		frame_cursor.write_variable_u64(self.connection_id).map_err(EncodeError::Io)?;
		frame_cursor.write_variable_u64(self.frame_id).map_err(EncodeError::Io)?;
		frame_cursor.write_u8(u8::from(self.reliability)).map_err(EncodeError::Io)?;
		self.headers.encode_headers(&mut frame_cursor).map_err(EncodeError::Io)?;
		self.segment.encode(&mut frame_cursor, cipher, self.frame_id)?;
		Ok(frame_cursor.position() as usize)
	}
}

#[cfg(test)]
pub mod tests {
	use crate::codec::cipher::Cipher;
	use crate::frame::headers::Header;
	use crate::frame::segment::Segment;
	use crate::frame::Frame;
	use crate::reliable::ack::header::AckHeader;

	const PRIVATE_KEY: &[u8] = &[
		0x29, 0xfa, 0x35, 0x60, 0x88, 0x45, 0xc6, 0xf9, 0xd8, 0xfe, 0x65, 0xe3, 0x22, 0x0e, 0x5b, 0x05, 0x03, 0x4a, 0xa0, 0x9f, 0x9e, 0x27, 0xad, 0x0f, 0x6c, 0x90, 0xa5, 0x73, 0xa8, 0x10, 0xe4, 0x94,
	];

	#[test]
	fn should_encode_decode_frame() {
		let mut frame = Frame::new(100, 55, true, Segment::default_with_body(&[1, 2, 3, 4, 6]));
		let key = PRIVATE_KEY.into();
		let mut cipher = Cipher::new(&key);
		frame.headers.add(Header::Ack(AckHeader::default()));
		frame.headers.add(Header::Ack(AckHeader::default()));

		let mut buffer = [0; 1024];
		let size = frame.encode(&mut cipher, &mut buffer).unwrap();
		let buffer = &buffer[0..size];
		let decoded_frame = Frame::decode(buffer, |_| Some(cipher.clone())).unwrap();

		assert_eq!(decoded_frame, decoded_frame);
	}
}
