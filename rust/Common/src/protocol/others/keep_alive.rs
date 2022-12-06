use crate::protocol::frame::output::OutFrame;
use std::ops::Sub;
use std::time::{Duration, Instant};

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

	pub fn build_frame(&mut self, _: &mut OutFrame, now: Instant) {
		self.last_send = Some(now);
	}
}

#[cfg(test)]
mod tests {
	use crate::protocol::frame::output::OutFrame;
	use std::ops::Add;
	use std::time::Instant;

	use crate::protocol::others::keep_alive::KeepAlive;

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
		let mut frame = OutFrame::new(1);
		handler.build_frame(&mut frame, now);
		assert!(!handler.contains_self_data(now));
		assert!(handler.contains_self_data(now.add(KeepAlive::INTERVAL)));
	}
}
