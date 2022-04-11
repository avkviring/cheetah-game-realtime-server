use std::collections::HashMap;
use std::sync::Mutex;

use prometheus::core::{Atomic, AtomicU64, GenericCounter, GenericGauge};
use prometheus::{IntCounter, IntGauge, Opts};
use tracing_subscriber::fmt::format;

pub struct GaugeByTagMeasures<P: Atomic> {
	name: String,
	help: String,
	tag_name: String,
	tools: Mutex<HashMap<String, GenericGauge<P>>>,
}

impl<P: 'static + Atomic> GaugeByTagMeasures<P> {
	pub fn new(name: &str, help: &str, tag_name: &str) -> Self {
		Self {
			name: name.into(),
			help: help.into(),
			tag_name: tag_name.into(),
			tools: Mutex::new(Default::default()),
		}
	}

	pub fn set(&self, tag_value: &str, value: P::T) {
		self.apply(tag_value, |g| {
			g.set(value);
		});
	}

	pub fn inc(&self, tag_value: &str) {
		self.apply(tag_value, |g| g.inc());
	}

	pub fn inc_by(&self, tag_value: &str, value: P::T) {
		self.apply(tag_value, |g| g.add(value));
	}

	pub fn dec(&self, tag_value: &str) {
		self.apply(tag_value, |g| g.dec());
	}

	fn apply<F>(&self, tag_value: &str, op: F)
	where
		F: Fn(&mut GenericGauge<P>) -> (),
	{
		let mut locked_tools = self.tools.lock();
		let locked_tools = locked_tools.as_mut().unwrap();
		let gauge = locked_tools.entry(tag_value.into()).or_insert_with(|| {
			let opts = Opts::new(self.name.clone(), self.help.as_str())
				.const_labels([(self.tag_name.clone(), tag_value.into())].into_iter().collect());
			let gauge = GenericGauge::<P>::with_opts(opts).unwrap();
			prometheus::default_registry().register(Box::new(gauge.clone())).unwrap();
			gauge
		});
		op(gauge);
	}
}
