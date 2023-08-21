use std::time::{Duration, Instant};

use prometheus::Histogram;
use prometheus::HistogramOpts;
use prometheus::IntGauge;
use prometheus::Opts;
use prometheus::Registry;
use prometheus_measures_exporter::measurer::create_and_register_measurer;

use crate::server::network::Network;
use crate::server::room_registry::Rooms;

///
/// Измерение параметров сервера - сохранение в prometheus
///
pub struct Measurer {
	room_count: IntGauge,
	member_count: IntGauge,
	object_count: IntGauge,
	income_command_count: IntGauge,
	outcome_command_count: IntGauge,
	income_frame_count: IntGauge,
	outcome_frame_count: IntGauge,
	cycle_time: Histogram,
}

impl Default for Measurer {
	fn default() -> Self {
		Measurer::new(prometheus::default_registry())
	}
}

impl Measurer {
	#[must_use]
	pub fn new(registry: &Registry) -> Self {
		Self {
			room_count: create_and_register_measurer::<_, _>(registry, Opts::new("room_count", "Room count")),
			member_count: create_and_register_measurer::<_, _>(registry, Opts::new("member_count", "Member count")),
			object_count: create_and_register_measurer::<_, _>(registry, Opts::new("object_count", "Object count")),
			income_command_count: create_and_register_measurer::<_, _>(registry, Opts::new("income_command_count", "Income command count")),
			outcome_command_count: create_and_register_measurer::<_, _>(registry, Opts::new("outcome_command_count", "Outcome command count")),
			income_frame_count: create_and_register_measurer::<_, _>(registry, Opts::new("income_frame_count", "Income frame count")),
			outcome_frame_count: create_and_register_measurer::<_, _>(registry, Opts::new("outcome_frame_count", "Outcome frame count")),
			cycle_time: Self::create_execution_time(registry),
		}
	}

	fn create_execution_time(registry: &Registry) -> Histogram {
		create_and_register_measurer(
			registry,
			HistogramOpts::new("cycle_time", "Server cycle time").buckets(vec![
				Duration::from_micros(5).as_secs_f64(),
				Duration::from_micros(50).as_secs_f64(),
				Duration::from_micros(100).as_secs_f64(),
				Duration::from_micros(500).as_secs_f64(),
				Duration::from_millis(1).as_secs_f64(),
				Duration::from_millis(5).as_secs_f64(),
				Duration::from_millis(50).as_secs_f64(),
				Duration::from_millis(100).as_secs_f64(),
				Duration::from_millis(500).as_secs_f64(),
			]),
		)
	}

	pub(crate) fn measure_cycle(&mut self, network_server: &Network, rooms_registry: &Rooms, start_cycle_time: &Instant) {
		self.measure_execution_time(start_cycle_time);
		self.measure_rooms(rooms_registry);
		self.measure_network_server(network_server);
	}

	fn measure_execution_time(&mut self, start_cycle_time: &Instant) {
		let cycle_time = start_cycle_time.elapsed();
		self.cycle_time.observe(cycle_time.as_secs_f64());
	}

	fn measure_network_server(&mut self, network_server: &Network) {
		self.income_command_count.set(network_server.income_command_count as i64);
		self.outcome_command_count.set(network_server.outcome_command_count as i64);
		self.income_frame_count.set(network_server.income_frame_count as i64);
		self.outcome_frame_count.set(network_server.outcome_frame_count as i64);
	}

	fn measure_rooms(&mut self, rooms_registry: &Rooms) {
		self.room_count.set(rooms_registry.rooms().len() as i64);
		let mut member_count = 0;
		let mut object_count = 0;
		for (_, room) in rooms_registry.rooms() {
			member_count += room.members.len();
			object_count += room.objects.len();
		}
		self.member_count.set(member_count as i64);
		self.object_count.set(object_count as i64);
	}
}
