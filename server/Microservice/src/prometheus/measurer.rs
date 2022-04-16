use prometheus::core::Collector;
use prometheus::Registry;

use crate::prometheus::{MeasureBuilder, ENABLE_PROMETHEUS};

///
/// Создание и регистрация prometheus измерителя
///
pub fn create_and_register_measurer<T: 'static, OPTS>(registry: &Registry, opts: OPTS) -> T
where
	T: Collector + MeasureBuilder<OPTS> + Clone,
{
	let measurer: T = MeasureBuilder::<OPTS>::build(opts);
	if *ENABLE_PROMETHEUS.lock().unwrap() {
		match registry.register(Box::new(measurer.clone())) {
			Ok(_) => {}
			Err(e) => {
				panic!("Enable register prometheus measurer {:?}", e)
			}
		};
	}
	measurer
}

#[cfg(test)]
mod test {
	use prometheus::proto::MetricFamily;
	use prometheus::{IntCounter, Opts, Registry};

	use crate::prometheus::measurer::create_and_register_measurer;
	use crate::prometheus::ENABLE_PROMETHEUS;

	#[test]
	pub fn test() {
		*ENABLE_PROMETHEUS.lock().unwrap() = true;
		let registry = Registry::new();
		let counter = create_and_register_measurer::<IntCounter, _>(
			&registry,
			Opts::new("name", "help").const_labels(vec![("label".to_string(), "value".to_string())].into_iter().collect()),
		);

		counter.inc();

		let metrics = registry.gather();
		assert_eq!(metrics.len(), 1);
		let metric_family: &MetricFamily = metrics.first().unwrap();
		assert_eq!(metric_family.get_name(), "name");
		assert_eq!(metric_family.get_help(), "help");
		let metric = &metric_family.get_metric()[0];
		assert_eq!(metric.get_counter().get_value(), 1.0);
		let label = metric.get_label().first().unwrap();
		assert_eq!(label.get_name(), "label");
		assert_eq!(label.get_value(), "value");
	}
}
