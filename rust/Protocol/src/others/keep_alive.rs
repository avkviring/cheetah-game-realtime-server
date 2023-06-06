use std::ops::Sub;
use std::time::{Duration, Instant};

use crate::frame::Frame;

///
/// Поддержание канала в открытом состоянии если нет прикладных команд
///
#[derive(Default, Debug)]
pub struct KeepAlive {
	last_send: Option<Instant>,
}

impl KeepAlive {
	// должно быть кратно меньше чем время разрыва соединения
	const INTERVAL: Duration = Duration::from_secs(1);

	#[must_use]
	pub fn contains_self_data(&self, now: Instant) -> bool {
		match self.last_send.as_ref() {
			None => true,
			Some(last_time) => now.sub(*last_time) >= KeepAlive::INTERVAL,
		}
	}

	pub fn build_frame(&mut self, _: &mut Frame, now: Instant) {
		self.last_send = Some(now);
	}
}

#[cfg(test)]
mod tests {
	use crate::frame::Frame;
	use crate::others::keep_alive::KeepAlive;
	use std::ops::Add;
	use std::time::Instant;

	#[test]
	pub(crate) fn should_send_first_time() {
		let handler = KeepAlive::default();
		let now = Instant::now();
		assert!(handler.contains_self_data(now));
	}

	#[test]
	pub(crate) fn should_timeout_after_send() {
		let mut handler = KeepAlive::default();
		let now = Instant::now();
		let mut frame = Frame::new(0, 1, true, Default::default());
		handler.build_frame(&mut frame, now);
		assert!(!handler.contains_self_data(now));
		assert!(handler.contains_self_data(now.add(KeepAlive::INTERVAL)));
	}
}
