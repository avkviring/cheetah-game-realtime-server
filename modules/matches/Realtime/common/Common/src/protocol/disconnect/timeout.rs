use std::ops::Sub;
use std::time::{Duration, Instant};
///
/// Если за определенное время не было входящих пакетов - считаем что связь разорвана
///
#[derive(Debug)]
pub struct DisconnectByTimeout {
	pub last_in_frame_time: Instant,
}

impl DisconnectByTimeout {
	pub const TIMEOUT: Duration = Duration::from_secs(10);
	pub fn new(now: &Instant) -> Self {
		Self { last_in_frame_time: *now }
	}
	pub fn on_frame_received(&mut self, now: &Instant) {
		self.last_in_frame_time = *now;
	}
	pub fn disconnected(&self, now: &Instant) -> bool {
		now.sub(self.last_in_frame_time) > DisconnectByTimeout::TIMEOUT
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::protocol::disconnect::timeout::DisconnectByTimeout;

	#[test]
	///
	/// После запуска - канал некоторые время считается открытым
	///
	pub fn should_not_disconnect_when_start() {
		let now = Instant::now();
		let handler = DisconnectByTimeout::new(&now);
		assert!(!handler.disconnected(&now));
	}

	///
	/// Разрыв связи через timeout после старта, если не было ни одного фрейма
	///
	#[test]
	pub fn should_disconnect_after_timeout() {
		let now = Instant::now();
		let handler = DisconnectByTimeout::new(&now);
		assert!(handler.disconnected(&now.add(DisconnectByTimeout::TIMEOUT).add(Duration::from_millis(1))));
	}

	///
	/// Если был пакет - то канал не закрыт определенное время после этого
	///
	#[test]
	pub fn should_not_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let mut handler = DisconnectByTimeout::new(&now);
		handler.on_frame_received(&now);
		assert!(!handler.disconnected(&now.add(DisconnectByTimeout::TIMEOUT - Duration::from_millis(1))));
	}

	///
	/// Если был пакет - то канал закрыт после таймаута
	///
	#[test]
	pub fn should_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let mut handler = DisconnectByTimeout::new(&now);
		handler.on_frame_received(&now);
		assert!(handler.disconnected(&now.add(DisconnectByTimeout::TIMEOUT + Duration::from_millis(1))));
	}
}
