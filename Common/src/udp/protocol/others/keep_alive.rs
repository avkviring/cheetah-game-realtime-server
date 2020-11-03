use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::udp::protocol::FrameBuilder;
use crate::udp::protocol::frame::Frame;

///
/// Поддержание канала в открытом состоянии если нет прикладных команд
///
#[derive(Default)]
pub struct KeepAlive {
	last_send: Option<Instant>
}


impl KeepAlive {
	const INTERVAL: Duration = Duration::from_secs(1);
}

impl FrameBuilder for KeepAlive {
	fn contains_self_data(&self, now: &Instant) -> bool {
		match self.last_send.as_ref() {
			None => {
				false
			}
			Some(last_time) => {
				now.sub(*last_time) >= KeepAlive::INTERVAL
			}
		}
	}
	
	fn build_frame(&mut self, _: &mut Frame, now: &Instant) {
		self.last_send = Option::Some(now.clone());
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::Instant;
	
	use crate::udp::protocol::frame::Frame;
	use crate::udp::protocol::FrameBuilder;
	use crate::udp::protocol::others::keep_alive::KeepAlive;
	
	#[test]
	pub fn should_send_first_time() {
		let handler = KeepAlive::default();
		let now = Instant::now();
		assert_eq!(handler.contains_self_data(&now), true);
	}
	
	#[test]
	pub fn should_timeout_after_send() {
		let mut handler = KeepAlive::default();
		let now = Instant::now();
		let mut frame = Frame::new(1);
		handler.build_frame(&mut frame, &now);
		assert_eq!(handler.contains_self_data(&now), false);
		assert_eq!(handler.contains_self_data(&now.add(KeepAlive::INTERVAL)), true);
	}
}