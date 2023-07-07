use std::io::Cursor;

use crate::codec::variable_int::{VariableIntReader, VariableIntWriter};
use crate::frame::FrameId;

///
/// Заголовок для указания факта повторной передачи данного фрейма
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RetransmitHeader {
	pub original_frame_id: FrameId,
}

impl RetransmitHeader {
	#[must_use]
	pub fn new(original_frame_id: FrameId) -> Self {
		Self { original_frame_id }
	}
	pub(crate) fn decode(input: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
		Ok(Self {
			original_frame_id: input.read_variable_u64()?,
		})
	}

	pub(crate) fn encode(&self, out: &mut Cursor<&mut [u8]>) -> std::io::Result<()> {
		out.write_variable_u64(self.original_frame_id)
	}
}
