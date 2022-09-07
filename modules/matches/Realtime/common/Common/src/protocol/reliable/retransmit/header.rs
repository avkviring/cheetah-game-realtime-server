use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::protocol::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::protocol::frame::FrameId;

///
/// Заголовок для указания факта повторной передачи данного фрейма
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RetransmitHeader {
	pub original_frame_id: FrameId,
	pub retransmit_count: u8,
}

impl RetransmitHeader {
	pub fn new(original_frame_id: FrameId, retransmit_count: u8) -> Self {
		Self {
			original_frame_id,
			retransmit_count,
		}
	}
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			original_frame_id: input.read_variable_u64()?,
			retransmit_count: input.read_u8()?,
		})
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.original_frame_id)?;
		out.write_u8(self.retransmit_count)
	}
}
