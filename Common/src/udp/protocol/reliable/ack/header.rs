use std::ops::{BitAnd, Shl};

use serde::{Deserialize, Serialize};

use crate::udp::protocol::frame::FrameId;

///
/// Подтверждение пакета
/// - содержит подтверждение для N фреймов, начиная от [start_frame_id]
/// - N зависит от [AskFrameHeader::CAPACITY]
///
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AckFrameHeader {
	///
	/// id подтверждаемого пакета
	///
	pub start_frame_id: FrameId,
	
	///
	/// Битовая маска для подтверждения следующих фреймов
	/// - каждый бит - +1 к [acked_frame_id]
	///
	frames: [u8; AckFrameHeader::CAPACITY / 8],
}

impl AckFrameHeader {
	///
	/// Максимальная разница между start_frame_id и frame_id
	/// Если разница меньше - то структура может сохранить frame_id
	///
	pub const CAPACITY: usize = 8 * 8;
	
	pub fn new(acked_frame_id: FrameId) -> Self {
		Self {
			start_frame_id: acked_frame_id,
			frames: [0; AckFrameHeader::CAPACITY / 8],
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
		if offset >= AckFrameHeader::CAPACITY {
			return false;
		}
		
		let byte_offset = offset / 8;
		let bit_offset = offset - byte_offset * 8;
		let byte = self.frames[byte_offset].clone();
		self.frames[byte_offset] = byte + 1.shl(bit_offset) as u8;
		true
	}
	
	pub fn get_frames(&self) -> Vec<u64> {
		let mut result = Vec::new();
		result.push(self.start_frame_id);
		for i in 0..AckFrameHeader::CAPACITY {
			let byte_offset = i / 8;
			let bit_offset = i - byte_offset * 8;
			let byte = self.frames[byte_offset].clone();
			if byte.bitand(1.shl(bit_offset) as u8) > 0 {
				result.push(self.start_frame_id + i as u64 + 1);
			}
		}
		
		result
	}
}


#[cfg(test)]
mod tests {
	use crate::udp::protocol::reliable::ack::header::AckFrameHeader;
	
	#[test]
	///
	/// Проверяем сохранение списка frame_id
	///
	pub fn should_store_frame_id() {
		let frame_first = 100;
		let mut header = AckFrameHeader::new(frame_first);
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
		let mut header = AckFrameHeader::new(frame_first);
		assert_eq!(header.store_frame_id(frame_first + AckFrameHeader::CAPACITY as u64 + 1), false)
	}
}