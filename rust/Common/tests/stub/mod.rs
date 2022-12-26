use std::ops::{Add, RangeInclusive};
use std::time::{Duration, Instant};

use rand::rngs::OsRng;
use rand::Rng;

use cheetah_common::protocol::frame::input::InFrame;
use cheetah_common::protocol::reliable::retransmit::RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC;
use cheetah_common::protocol::Protocol;

#[derive(Default)]
pub struct Channel {
	reliable_percents: Vec<(RangeInclusive<u64>, f64)>,
}

impl Channel {
	pub fn cycle(&mut self, count: usize, peer_a: &mut Protocol, peer_b: &mut Protocol) {
		let mut now = Instant::now();

		for i in 0..count {
			let frame_a = peer_a.build_next_frame(now);
			if let Some(frame_a) = frame_a {
				if self.allow(i as u64) {
					let commands = frame_a.get_commands().cloned().collect::<Vec<_>>();
					peer_b.on_frame_received(&InFrame::new(frame_a.frame_id, frame_a.headers, commands), now);
				}
			}

			let frame_b = peer_b.build_next_frame(now);
			if let Some(frame_b) = frame_b {
				if self.allow(i as u64) {
					let commands = frame_b.get_commands().cloned().collect::<Vec<_>>();
					peer_a.on_frame_received(&InFrame::new(frame_b.frame_id, frame_b.headers, commands), now);
				}
			}

			now = now.add(Duration::from_secs_f64(RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC));
		}
	}

	pub fn add_reliable_percent(&mut self, range: RangeInclusive<u64>, transfered_percent: f64) {
		self.reliable_percents.push((range, transfered_percent));
	}

	#[must_use]
	pub fn allow(&self, position: u64) -> bool {
		let find = self.reliable_percents.iter().find_map(|(range, percent)| range.contains(&position).then(|| OsRng.gen_bool(*percent)));
		find.unwrap_or(true)
	}
}
