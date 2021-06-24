use std::ops::{Add, RangeInclusive};
use std::time::{Duration, Instant};

use rand::rngs::OsRng;
use rand::Rng;

use cheetah_relay_common::protocol::relay::RelayProtocol;

#[derive(Default)]
pub struct Channel {
	reliable_percents: Vec<(RangeInclusive<u64>, f64)>,
}

impl Channel {
	pub fn cycle(&mut self, count: usize, peer_a: &mut RelayProtocol, peer_b: &mut RelayProtocol) {
		let mut now = Instant::now();

		for i in 0..count {
			let frame_a = peer_a.build_next_frame(&now);
			if let Some(frame_a) = frame_a {
				if self.allow(i as u64) {
					peer_b.on_frame_received(frame_a, &now)
				}
			}

			let frame_b = peer_b.build_next_frame(&now);
			if let Some(frame_b) = frame_b {
				if self.allow(i as u64) {
					peer_a.on_frame_received(frame_b, &now)
				}
			}

			now = now.add(Duration::from_millis(100));
		}
	}

	pub fn add_reliable_percent(&mut self, range: RangeInclusive<u64>, transfered_percent: f64) {
		self.reliable_percents.push((range, transfered_percent));
	}

	pub fn allow(&self, position: u64) -> bool {
		let find = self.reliable_percents.iter().find_map(|(range, percent)| {
			if range.contains(&position) {
				Option::Some(OsRng.gen_bool(*percent))
			} else {
				Option::None
			}
		});
		match find {
			None => true,
			Some(allow) => allow,
		}
	}
}