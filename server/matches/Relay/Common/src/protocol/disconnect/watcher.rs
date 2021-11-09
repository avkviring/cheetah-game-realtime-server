use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::protocol::frame::Frame;
use crate::protocol::{DisconnectedStatus, FrameReceivedListener};

///
/// Если за определенное время не было входящих пакетов - считаем что связь разорвана
///
#[derive(Debug)]
pub struct DisconnectWatcher {
	pub last_in_frame_time: Instant,
}

impl DisconnectWatcher {
	pub const TIMEOUT: Duration = Duration::from_secs(10);
	pub fn new(now: &Instant) -> Self {
		Self {
			last_in_frame_time: *now,
		}
	}
}

impl FrameReceivedListener for DisconnectWatcher {
	fn on_frame_received(&mut self, _: &Frame, now: &Instant) {
		self.last_in_frame_time = *now;
	}
}

impl DisconnectedStatus for DisconnectWatcher {
	fn disconnected(&self, now: &Instant) -> bool {
		now.sub(self.last_in_frame_time) > DisconnectWatcher::TIMEOUT
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::protocol::disconnect::watcher::DisconnectWatcher;
	use crate::protocol::frame::Frame;
	use crate::protocol::{DisconnectedStatus, FrameReceivedListener};

	#[test]
	///
	/// После запуска - канал некоторые время считается открытым
	///
	pub fn should_not_disconnect_when_start() {
		let now = Instant::now();
		let handler = DisconnectWatcher::new(&now);
		assert_eq!(handler.disconnected(&now), false);
	}

	///
	/// Разрыв связи через timeout после старта, если не было ни одного фрейма
	///
	#[test]
	pub fn should_disconnect_after_timeout() {
		let now = Instant::now();
		let handler = DisconnectWatcher::new(&now);
		assert_eq!(
			handler.disconnected(&now.add(DisconnectWatcher::TIMEOUT).add(Duration::from_millis(1))),
			true
		);
	}

	///
	/// Если был пакет - то канал не закрыт определенное время после этого
	///
	#[test]
	pub fn should_not_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let mut handler = DisconnectWatcher::new(&now);
		let frame = Frame::new(0);
		handler.on_frame_received(&frame, &now);
		assert_eq!(
			handler.disconnected(&now.add(DisconnectWatcher::TIMEOUT - Duration::from_millis(1))),
			false
		);
	}

	///
	/// Если был пакет - то канал закрыт после таймаута
	///
	#[test]
	pub fn should_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let mut handler = DisconnectWatcher::new(&now);
		let frame = Frame::new(0);
		handler.on_frame_received(&frame, &now);
		assert_eq!(
			handler.disconnected(&now.add(DisconnectWatcher::TIMEOUT + Duration::from_millis(1))),
			true
		);
	}
}
