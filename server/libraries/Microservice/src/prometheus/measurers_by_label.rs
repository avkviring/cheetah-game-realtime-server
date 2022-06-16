use std::collections::HashMap;
use std::hash::Hash;

use prometheus::core::Collector;
use prometheus::{Histogram, HistogramOpts, IntCounter, Opts, Registry};

use crate::prometheus::{MeasureBuilder, ENABLE_PROMETHEUS};

pub type IntCounterMeasurersByLabel<K> = MeasurersByLabel<K, IntCounter, Opts>;
pub type HistogramMeasurersByLabel<K> = MeasurersByLabel<K, Histogram, HistogramOpts>;

///
/// Доступ к prometheus измерителям по набору меток
///
pub struct MeasurersByLabel<K, T, OPTS>
where
	T: Collector,
{
	registry: Registry,
	tools: HashMap<K, T>,
	opts_factory: Box<LabelFactoryFactory<K, OPTS>>,
}
pub type LabelFactoryFactory<K, OPTS> = dyn Fn(&K) -> OPTS;

impl<K, T: 'static, OPTS> MeasurersByLabel<K, T, OPTS>
where
	K: Eq + Hash + Clone,
	T: Collector + MeasureBuilder<OPTS> + Clone,
{
	pub fn new(registry: &Registry, opts_factory: Box<LabelFactoryFactory<K, OPTS>>) -> Self {
		Self {
			registry: registry.clone(),
			tools: Default::default(),
			opts_factory,
		}
	}

	pub fn measurer(&mut self, key: &K) -> &mut T {
		if self.tools.get(key).is_none() {
			let opts = (self.opts_factory)(key);
			let measurer: T = MeasureBuilder::<OPTS>::build(opts);
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

#[cfg(test)]
mod test {
	use prometheus::proto::MetricFamily;
	use prometheus::{IntCounter, Opts, Registry};

	use crate::prometheus::measurers_by_label::MeasurersByLabel;
	use crate::prometheus::ENABLE_PROMETHEUS;

	#[test]
	pub fn test() {
		*ENABLE_PROMETHEUS.lock().unwrap() = true;
		let registry = Registry::new();
		let mut measures = MeasurersByLabel::<u8, IntCounter, Opts>::new(
			&registry,
			Box::new(|key| {
				Opts::new("name", "help").const_labels(
					vec![("label".to_string(), key.to_string())]
						.into_iter()
						.collect(),
				)
			}),
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
