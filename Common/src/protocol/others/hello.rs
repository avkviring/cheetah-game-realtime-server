use std::time::Instant;

use crate::protocol::frame::Frame;
use crate::protocol::frame::headers::Header;
use crate::protocol::FrameBuilder;

///
/// Однократная отправка HELLO
///
#[derive(Default, Debug)]
pub struct HelloSender {
	sent: bool
}

impl FrameBuilder for HelloSender {
	fn contains_self_data(&self, _: &Instant) -> bool {
		!self.sent
	}
	
	fn build_frame(&mut self, frame: &mut Frame, _: &Instant) {
		self.sent = true;
		frame.headers.add(Header::Hello)
	}
}