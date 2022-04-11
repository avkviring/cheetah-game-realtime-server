use prometheus::{IntCounter, Opts};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct IntCounterMeasures {
	name: String,
	help: String,
	tag_name: String,
	tools: Mutex<HashMap<String, IntCounter>>,
}

impl IntCounterMeasures {
	pub fn new(name: &str, help: &str, tag_name: &str) -> Self {
		Self {
			name: name.into(),
			help: help.into(),
			tag_name: tag_name.into(),
			tools: Mutex::new(Default::default()),
		}
	}

	fn incr(&self, tag_value: &str) {
		let mut locked_tools = self.tools.lock();
		let locked_tools = locked_tools.as_mut().unwrap();
		let counter = locked_tools.entry(tag_value.into()).or_insert_with(|| {
			let opts = Opts::new(self.name.clone(), self.help.as_str())
				.const_labels([(self.tag_name.clone(), tag_value.into())].into_iter().collect());
			let counter = IntCounter::with_opts(opts).unwrap();
			prometheus::default_registry().register(Box::new(counter.clone())).unwrap();
			counter
		});
		counter.inc();
	}
}
