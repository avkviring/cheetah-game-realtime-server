use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

use prometheus::core::{Atomic, Collector, GenericGauge};
use prometheus::{IntCounter, Opts, Registry};

use crate::prometheus::ENABLE_PROMETHEUS;

///
/// Доступ к prometheus измерителям по набору меток
///
pub struct MeasurersByLabel<K, T>
where
	T: Collector,
{
	registry: Registry,
	name: String,
	help: String,
	tools: HashMap<K, T>,
	label_factory: Box<LabelFactoryFactory<K>>,
}
pub type LabelFactoryFactory<K> = dyn Fn(&K) -> Vec<(&'static str, String)>;

impl<K, T: 'static> MeasurersByLabel<K, T>
where
	K: Eq + Hash + Clone,
	T: Collector + CollectorBuilder + Clone,
{
	pub fn new(name: &str, help: &str, registry: Registry, label_factory: Box<LabelFactoryFactory<K>>) -> Self {
		Self {
			registry,
			name: name.into(),
			help: help.into(),
			tools: Default::default(),
			label_factory,
		}
	}

	pub fn measurer(&mut self, key: &K) -> &mut T {
		if let None = self.tools.get(key) {
			let labels_map: HashMap<String, String> = (self.label_factory)(key)
				.iter()
				.map(|(k, v)| (k.to_string(), v.to_string()))
				.collect();
			let opts = Opts::new(self.name.clone(), self.help.as_str()).const_labels(labels_map);
			let measurer: T = CollectorBuilder::build(opts);
			if *ENABLE_PROMETHEUS.lock().unwrap() {
				match self.registry.register(Box::new(measurer.clone())) {
					Ok(_) => {}
					Err(e) => {
						panic!("Enable register prometheus measurer {:?}", e)
					}
				};
			}

			self.tools.insert(key.clone(), measurer);
		}

		self.tools.get_mut(key).unwrap()
	}
}
pub trait CollectorBuilder {
	fn build(source: Opts) -> Self;
}
impl CollectorBuilder for IntCounter {
	fn build(opts: Opts) -> Self {
		IntCounter::with_opts(opts).unwrap()
	}
}

impl<P> CollectorBuilder for GenericGauge<P>
where
	P: Atomic,
{
	fn build(source: Opts) -> Self {
		GenericGauge::<P>::with_opts(source).unwrap()
	}
}

#[cfg(test)]
mod test {
	use prometheus::proto::MetricFamily;
	use prometheus::{IntCounter, Registry};

	use crate::prometheus::measurers::{MeasurersByLabel, ENABLE_PROMETHEUS};

	#[test]
	pub fn test() {
		*ENABLE_PROMETHEUS.lock().unwrap() = true;
		let registry = Registry::new();
		let mut measures = MeasurersByLabel::<u8, IntCounter>::new(
			"name",
			"help",
			registry.clone(),
			Box::new(|key| vec![("label", key.to_string())]),
		);

		let counter = measures.measurer(&10);
		counter.inc();
		let counter = measures.measurer(&10);
		counter.inc();

		let metrics = registry.gather();
		assert_eq!(metrics.len(), 1);
		let metric_family: &MetricFamily = metrics.first().unwrap();
		assert_eq!(metric_family.get_name(), "name");
		assert_eq!(metric_family.get_help(), "help");
		let metric = &metric_family.get_metric()[0];
		assert_eq!(metric.get_counter().get_value(), 2.0);
		let label = metric.get_label().first().unwrap();
		assert_eq!(label.get_name(), "label");
		assert_eq!(label.get_value(), "10");
	}
}
