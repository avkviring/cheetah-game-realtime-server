use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::udp::protocol::{DisconnectedStatus, FrameReceivedListener};
use crate::udp::protocol::frame::Frame;

///
/// Если за определенное время не было входящих пакетов - считаем что связь разорвана
///
#[derive(Default, Debug)]
pub struct DisconnectWatcher {
	pub last_in_frame_time: Option<Instant>
}

impl DisconnectWatcher {
	pub const TIMEOUT: Duration = Duration::from_secs(10);
}

impl FrameReceivedListener for DisconnectWatcher {
	fn on_frame_received(&mut self, _: &Frame, now: &Instant) {
		self.last_in_frame_time = Option::Some(now.clone());
	}
}

impl DisconnectedStatus for DisconnectWatcher {
	fn disconnected(&mut self, now: &Instant) -> bool {
		match self.last_in_frame_time {
			None => {
				false
			}
			Some(ref prev) => {
				now.sub(*prev) > DisconnectWatcher::TIMEOUT
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};
	
	use crate::udp::protocol::{DisconnectedStatus, FrameReceivedListener};
	use crate::udp::protocol::disconnect::watcher::DisconnectWatcher;
	use crate::udp::protocol::frame::Frame;
	
	///
						/// Если не было ни одного входящего фрейма - то канал не закрыт (но и не открыт)
						///
	#[test]
	pub fn should_not_disconnect_when_zero_in_frame() {
		let mut handler = DisconnectWatcher::default();
		let now = Instant::now();
		assert_eq!(handler.disconnected(&now), false);
	}
	
	///
	/// Если был пакет - то канал не закрыт определенное время после этого
	///
	#[test]
	pub fn should_not_disconnect_when_not_timeout_after_frame() {
		let mut handler = DisconnectWatcher::default();
		let now = Instant::now();
		let frame = Frame::new(0);
		handler.on_frame_received(&frame, &now);
		assert_eq!(handler.disconnected(&now.add(DisconnectWatcher::TIMEOUT - Duration::from_millis(1))), false);
	}
	
	///
	/// Если был пакет - то канал закрыт после таймаута
	///
	#[test]
	pub fn should_disconnect_when_not_timeout_after_frame() {
		let mut handler = DisconnectWatcher::default();
		let now = Instant::now();
		let frame = Frame::new(0);
		handler.on_frame_received(&frame, &now);
		assert_eq!(handler.disconnected(&now.add(DisconnectWatcher::TIMEOUT + Duration::from_millis(1))), true);
	}
}

