use std::time::Instant;

use crate::udp::protocol::frame::Frame;
use crate::udp::protocol::frame::headers::Header;
use crate::udp::protocol::FrameBuilder;

///
/// Однократная отправка HELLO
///
#[derive(Default)]
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