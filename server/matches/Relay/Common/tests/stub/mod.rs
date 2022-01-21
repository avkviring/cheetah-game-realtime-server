use std::ops::{Add, RangeInclusive};
use std::time::{Duration, Instant};

use cheetah_matches_relay_common::protocol::reliable::retransmit::RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC;
use cheetah_matches_relay_common::protocol::Protocol;
use rand::rngs::OsRng;
use rand::Rng;

#[derive(Default)]
pub struct Channel {
	reliable_percents: Vec<(RangeInclusive<u64>, f64)>,
}

impl Channel {
	pub fn cycle(&mut self, count: usize, peer_a: &mut Protocol, peer_b: &mut Protocol) {
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

			now = now.add(Duration::from_secs_f64(RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC));
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
		find.unwrap_or(true)
	}
}
