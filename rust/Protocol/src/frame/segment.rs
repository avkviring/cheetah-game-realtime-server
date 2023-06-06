pub const SEGMENT_SIZE: usize = 256;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Segment {
	pub packet_id: u64,
	pub count_segments: u8,
	pub current_segment: u8,
	pub body_size: usize,
	pub body: [u8; SEGMENT_SIZE],
}

impl Segment {
	pub(crate) fn new(packet_id: u64, count_segments: u8, current_segment: u8, body: &[u8]) -> Self {
		let mut result = Self {
			packet_id,
			count_segments,
			current_segment,
			body_size: body.len(),
			body: [0; SEGMENT_SIZE],
		};
		result.body[0..result.body_size].copy_from_slice(body);
		result
	}
}

impl Default for Segment {
	fn default() -> Self {
		Segment::new(1, 1, 0, &[1, 2, 3])
	}
}

#[cfg(test)]
impl Segment {
	pub(crate) fn default_with_body(body: &[u8]) -> Segment {
		Segment::new(1, 1, 0, body)
	}
}
