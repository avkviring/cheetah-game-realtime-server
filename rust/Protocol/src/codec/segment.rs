use std::io::{Cursor, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::codec::cipher::Cipher;
use crate::codec::compress::{packet_compress, packet_decompress};
use crate::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::codec::{DecodeError, EncodeError};
pub use crate::frame::segment::Segment;

impl Segment {
	pub(crate) fn decode<'a>(mut cursor: Cursor<&'a [u8]>, mut cipher: Cipher<'a>, frame_id: u64) -> Result<Segment, DecodeError> {
		let packet_id = cursor.read_variable_u64()?;
		let count_segments = cursor.read_u8()?;
		let current_segment = cursor.read_u8()?;

		let position = cursor.position();
		let data = cursor.into_inner();
		let nonce = frame_id.to_be_bytes();
		let ad = &data[0..position as usize];

		let mut vec: heapless::Vec<u8, 4096> = heapless::Vec::new();
		vec.extend_from_slice(&data[position as usize..data.len()]).map_err(|_| DecodeError::HeaplessError)?;
		cipher.decrypt(&mut vec, ad, nonce).map_err(DecodeError::DecryptedError)?;

		let mut body = [0; 4096];
		let decompressed_size = packet_decompress(&vec, body.as_mut_slice())?;
		Ok(Segment::new(packet_id, count_segments, current_segment, &body[0..decompressed_size]))
	}

	pub(crate) fn encode<'a>(&self, cursor: &mut Cursor<&'a mut [u8]>, cipher: &mut Cipher, frame_id: u64) -> Result<(), EncodeError> {
		cursor.write_variable_u64(self.packet_id)?;
		cursor.write_u8(self.count_segments)?;
		cursor.write_u8(self.current_segment)?;

		let data = cursor.get_ref();
		let nonce = frame_id.to_be_bytes();
		let ad = &data[0..cursor.position() as usize];

		let data = &self.body[0..self.body_size];
		let mut buffer: heapless::Vec<u8, 4096> = heapless::Vec::new();
		unsafe {
			buffer.set_len(4096);
		}
		let compressed_size = packet_compress(data, &mut buffer)?;
		unsafe {
			buffer.set_len(compressed_size);
		}

		cipher.encrypt(&mut buffer, ad, nonce).map_err(EncodeError::EncryptedError)?;
		cursor.write_all(&buffer).unwrap();
		Ok(())
	}
}
