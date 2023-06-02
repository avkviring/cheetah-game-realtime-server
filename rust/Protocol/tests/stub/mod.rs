use std::collections::VecDeque;
use std::ops::{Add, RangeInclusive};
use std::time::{Duration, Instant};

use rand::rngs::OsRng;
use rand::Rng;

use cheetah_protocol::frame::FRAME_BODY_CAPACITY;
use cheetah_protocol::reliable::retransmit::RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC;
use cheetah_protocol::{InputDataHandler, OutputDataProducer, Protocol};

#[derive(Default)]
pub struct StubDataRecvHandler {
	pub size_recv: usize,
}

impl InputDataHandler for StubDataRecvHandler {
	fn on_input_data(&mut self, data: &[u8]) {
		self.size_recv += data.len();
	}
}

#[derive(Default)]
pub struct StubDataSource {
	items: VecDeque<Vec<u8>>,
}

impl StubDataSource {
	pub(crate) fn add(&mut self, data: &[u8]) {
		self.items.push_back(data.into());
	}
}

impl OutputDataProducer for StubDataSource {
	fn contains_output_data(&self) -> bool {
		!self.items.is_empty()
	}

	fn get_output_data(&mut self, buffer: &mut [u8; FRAME_BODY_CAPACITY]) -> (usize, bool) {
		match self.items.pop_front() {
			None => (0, false),
			Some(source) => {
				buffer[0..source.len()].copy_from_slice(source.as_slice());
				(source.len(), true)
			}
		}
	}
}

#[derive(Default)]
pub struct Channel {
	reliable_percents: Vec<(RangeInclusive<u64>, f64)>,
}

impl Channel {
	pub fn cycle<DRH, DS>(&mut self, count: usize, peer_a: &mut Protocol<DRH, DS>, peer_b: &mut Protocol<DRH, DS>)
	where
		DRH: InputDataHandler,
		DS: OutputDataProducer,
	{
		let mut now = Instant::now();

		for i in 0..count {
			let frame_a = peer_a.build_next_frame(now);
			if let Some(frame_a) = frame_a {
				if self.allow(i as u64) {
					peer_b.on_frame_received(&frame_a, now);
				}
			}

			let frame_b = peer_b.build_next_frame(now);
			if let Some(frame_b) = frame_b {
				if self.allow(i as u64) {
					peer_a.on_frame_received(&frame_b, now);
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
