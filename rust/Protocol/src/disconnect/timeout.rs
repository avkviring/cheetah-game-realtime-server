use std::ops::Sub;
use std::time::{Duration, Instant};

///
/// Если за определенное время не было входящих пакетов - считаем что связь разорвана
///
#[derive(Debug)]
pub struct DisconnectByTimeout {
	pub last_in_frame_time: Instant,
	pub timeout: Duration,
}

impl DisconnectByTimeout {
	#[must_use]
	pub fn new(now: Instant, timeout: Duration) -> Self {
		Self { last_in_frame_time: now, timeout }
	}
	pub fn on_frame_received(&mut self, now: Instant) {
		self.last_in_frame_time = now;
	}
	#[must_use]
	pub fn is_disconnected(&self, now: Instant) -> bool {
		now.sub(self.last_in_frame_time) > self.timeout
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;
	use std::time::{Duration, Instant};

	use crate::disconnect::timeout::DisconnectByTimeout;

	#[test]
	///
	/// После запуска - канал некоторые время считается открытым
	///
	pub(crate) fn should_not_disconnect_when_start() {
		let now = Instant::now();
		let handler = DisconnectByTimeout::new(now, Duration::from_millis(1));
		assert!(!handler.is_disconnected(now));
	}

	///
	/// Разрыв связи через timeout после старта, если не было ни одного фрейма
	///
	#[test]
	pub(crate) fn should_disconnect_after_timeout() {
		let now = Instant::now();
		let timeout = Duration::from_millis(100);
		let handler = DisconnectByTimeout::new(now, timeout.clone());
		assert!(handler.is_disconnected(now.add(timeout).add(Duration::from_millis(1))));
	}

	///
	/// Если был пакет - то канал не закрыт определенное время после этого
	///
	#[test]
	pub(crate) fn should_not_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let timeout = Duration::from_millis(100);
		let mut handler = DisconnectByTimeout::new(now, timeout.clone());
		handler.on_frame_received(now);
		assert!(!handler.is_disconnected(now.add(timeout - Duration::from_millis(1))));
	}

	///
	/// Если был пакет - то канал закрыт после таймаута
	///
	#[test]
	pub(crate) fn should_disconnect_when_not_timeout_after_frame() {
		let now = Instant::now();
		let timeout = Duration::from_millis(100);
		let mut handler = DisconnectByTimeout::new(now, timeout.clone());
		handler.on_frame_received(now);
		assert!(handler.is_disconnected(now.add(timeout + Duration::from_millis(1))));
	}
}
