use std::collections::VecDeque;
use std::ops::{Add, RangeInclusive};
use std::time::{Duration, Instant};

use prometheus::{Histogram, HistogramOpts, IntCounter};
use rand::rngs::OsRng;
use rand::Rng;

use cheetah_protocol::frame::Frame;
use cheetah_protocol::reliable::retransmit::RETRANSMIT_DEFAULT_ACK_TIMEOUT_IN_SEC;
use cheetah_protocol::{InputDataHandler, OutputDataProducer, Protocol};

#[derive(Default)]
pub struct StubInputDataHandler {
	pub items: Vec<Vec<u8>>,
	pub size_recv: usize,
}

impl InputDataHandler for StubInputDataHandler {
	fn on_input_data(&mut self, data: &[u8]) {
		self.items.push(data.into());
		self.size_recv += data.len();
	}
}

#[derive(Default)]
pub struct StubOutputDataProducer {
	items: VecDeque<Vec<u8>>,
}

impl StubOutputDataProducer {
	pub fn add(&mut self, data: &[u8]) {
		self.items.push_back(data.into());
	}
}

impl OutputDataProducer for StubOutputDataProducer {
	fn contains_output_data(&self) -> bool {
		!self.items.is_empty()
	}

	fn get_output_data(&mut self, packet: &mut [u8]) -> (usize, bool) {
		match self.items.pop_front() {
			None => (0, false),
			Some(source) => {
				packet[0..source.len()].copy_from_slice(source.as_slice());
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
		let mut frames: VecDeque<Frame> = Default::default();

		for i in 0..count {
			frames.clear();
			peer_a.collect_out_frames(now, &mut frames);
			for frame in frames.iter() {
				if self.allow(i as u64) {
					peer_b.on_frame_received(frame, now);
				}
			}

			frames.clear();
			peer_b.collect_out_frames(now, &mut frames);
			for frame in frames.iter() {
				if self.allow(i as u64) {
					peer_a.on_frame_received(frame, now);
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

pub fn create_protocol() -> Protocol<StubInputDataHandler, StubOutputDataProducer> {
	Protocol::<StubInputDataHandler, StubOutputDataProducer>::new(
		Default::default(),
		Default::default(),
		0,
		Instant::now(),
		Instant::now(),
		IntCounter::new("name", "help").unwrap().local(),
		Histogram::with_opts(HistogramOpts::new("name", "help")).unwrap().local(),
	)
}
