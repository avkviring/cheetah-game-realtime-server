use std::io::{Cursor, Read, Write};
use std::ops::{BitAnd, Shl};

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::FrameId;

///
/// Подтверждение пакета
/// - содержит подтверждение для N фреймов, начиная от [start_frame_id]
/// - N зависит от [AskFrameHeader::CAPACITY]
///
#[derive(Debug, PartialEq, Clone)]
pub struct AckHeader {
	///
	/// id подтверждаемого пакета
	///
	pub start_frame_id: FrameId,

	///
	/// Битовая маска для подтверждения следующих фреймов
	/// - каждый бит - +1 к [acked_frame_id]
	///
	pub(crate) frames: [u8; AckHeader::CAPACITY / 8],
}

impl AckHeader {
	///
	/// Максимальная разница между start_frame_id и frame_id
	/// Если разница меньше - то структура может сохранить frame_id
	///
	pub const CAPACITY: usize = 8 * 8;

	pub fn new(acked_frame_id: FrameId) -> Self {
		Self {
			start_frame_id: acked_frame_id,
			frames: [0; AckHeader::CAPACITY / 8],
		}
	}

	///
	/// Сохранить frame_id
	/// - false если сохранение фреймы не возможно [AskFrameHeader::CAPACITY]
	///
	pub fn store_frame_id(&mut self, frame_id: u64) -> bool {
		if frame_id < self.start_frame_id {
			return false;
		}
		let offset = (frame_id - self.start_frame_id - 1) as usize;
		if offset >= AckHeader::CAPACITY {
			return false;
		}

		let byte_offset = offset / 8;
		let bit_offset = offset - byte_offset * 8;
		let byte = self.frames[byte_offset];
		self.frames[byte_offset] = byte + 1.shl(bit_offset) as u8;
		true
	}

	pub fn get_frames(&self) -> Vec<u64> {
		let mut result = vec![self.start_frame_id];
		for i in 0..AckHeader::CAPACITY {
			let byte_offset = i / 8;
			let bit_offset = i - byte_offset * 8;
			let byte = self.frames[byte_offset];
			if byte.bitand(1.shl(bit_offset) as u8) > 0 {
				result.push(self.start_frame_id + i as u64 + 1);
			}
		}

		result
	}

	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let mut result = Self {
			start_frame_id: input.read_variable_u64()?,
			frames: Default::default(),
		};
		input.read_exact(&mut result.frames)?;
		Ok(result)
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.start_frame_id)?;
		out.write_all(&self.frames)
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::reliable::ack::header::AckHeader;

	#[test]
	///
	/// Проверяем сохранение списка frame_id
	///
	pub fn should_store_frame_id() {
		let frame_first = 100;
		let mut header = AckHeader::new(frame_first);
		let offset = vec![1, 2, 3, 4, 7, 9, 15];
		offset.iter().for_each(|i| {
			header.store_frame_id(frame_first + i);
		});

		let frames = header.get_frames();
		assert_eq!(frames[0], 100);
		let mut frame_index = 1;
		offset.iter().for_each(|i| {
			assert_eq!(frames[frame_index], 100 + i);
			frame_index += 1;
		});
	}

	#[test]
	///
	/// Проверяем сохранение большего количества фреймов чем емкость хранилища
	///
	pub fn should_store_frame_fail_if_not_enough_capacity() {
		let frame_first = 100;
		let mut header = AckHeader::new(frame_first);
		assert!(!header.store_frame_id(frame_first + AckHeader::CAPACITY as u64 + 1))
	}
}
