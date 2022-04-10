use std::io::Cursor;
use std::slice::Iter;

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::FrameId;

///
/// Подтверждение пакета
/// - содержит подтверждение для N фреймов, начиная от [start_frame_id]
/// - N зависит от [AskFrameHeader::CAPACITY]
///
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AckHeader {
	frame_ids: heapless::Vec<FrameId, 20>,
}

impl AckHeader {
	pub fn add_frame_id(&mut self, frame_id: FrameId) {
		self.frame_ids.push(frame_id).unwrap();
	}

	pub fn get_frames(&self) -> Iter<FrameId> {
		self.frame_ids.iter()
	}

	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		let size = input.read_variable_u64()?;
		let mut result = Self::default();
		for _ in 0..size {
			match result.frame_ids.push(input.read_variable_u64()?) {
				Ok(_) => {}
				Err(_) => {
					tracing::error!("AckHeader decode error - overflow frame_ids",);
				}
			}
		}
		Ok(result)
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.frame_ids.len() as u64)?;
		for frame_id in self.get_frames() {
			out.write_variable_u64(*frame_id)?;
		}
		Ok(())
	}

	pub fn is_full(&self) -> bool {
		self.frame_ids.is_full()
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::frame::FrameId;
	use crate::protocol::reliable::ack::header::AckHeader;

	#[test]
	///
	/// Проверяем сохранение списка frame_id
	///
	pub fn should_store_frame_id() {
		let mut header = AckHeader::default();
		let originals = vec![1, 2, 3, 4, 7, 9, 15];
		originals.iter().for_each(|i| {
			header.add_frame_id(*i as FrameId);
		});

		let actual = header.get_frames().as_slice();
		assert_eq!(originals, actual);
	}
}
