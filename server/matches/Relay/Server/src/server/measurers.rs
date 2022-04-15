use std::time::Duration;

use prometheus::{Histogram, HistogramOpts, IntCounter, IntGauge, Opts};

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_microservice::prometheus::measurers::{LabelFactoryFactory, MeasurersByLabel};

pub type HeaplessStatisticString = heapless::String<50>;

///
/// Измерение параметров сервера - сохранение в prometheus
///
pub struct ServerMeasurers {
	room_count: MeasurersByLabel<String, IntGauge, Opts>,
	member_count: MeasurersByLabel<String, IntGauge, Opts>,
	object_count: MeasurersByLabel<String, IntGauge, Opts>,
	income_command_count: MeasurersByLabel<(Option<FieldType>, Option<FieldId>, HeaplessStatisticString), IntCounter, Opts>,
	outcome_command_count: MeasurersByLabel<(Option<FieldType>, Option<FieldId>, HeaplessStatisticString), IntCounter, Opts>,
	execution_time: MeasurersByLabel<(HeaplessStatisticString, Option<FieldId>), Histogram, HistogramOpts>,
}

impl ServerMeasurers {
	pub fn new() -> Self {
		let registry = &prometheus::default_registry();
		Self {
			room_count: MeasurersByLabel::new(
				registry,
				Box::new(|template| {
					Opts::new("room_count", "room count")
						.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
				}),
			),
			member_count: MeasurersByLabel::new(
				registry,
				Box::new(|template| {
					Opts::new("member_count", "member count")
						.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
				}),
			),
			object_count: MeasurersByLabel::new(
				registry,
				Box::new(|template| {
					Opts::new("object_count", "object count")
						.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
				}),
			),
			income_command_count: MeasurersByLabel::new(
				registry,
				Self::measurer_label_factory("income_command_counter", "Income command counter"),
			),
			outcome_command_count: MeasurersByLabel::new(
				registry,
				Self::measurer_label_factory("outcome_command_counter", "Outcome command counter"),
			),
			execution_time: MeasurersByLabel::new(
				registry,
				Box::new(|(command, field_id)| {
					HistogramOpts::new("command_execution_time", "command execution time")
						.buckets(vec![
							Duration::from_nanos(5).as_secs_f64(),
							Duration::from_nanos(10).as_secs_f64(),
							Duration::from_nanos(100).as_secs_f64(),
							Duration::from_nanos(500).as_secs_f64(),
							Duration::from_millis(1).as_secs_f64(),
							Duration::from_millis(5).as_secs_f64(),
							Duration::from_millis(10).as_secs_f64(),
							Duration::from_millis(50).as_secs_f64(),
						])
						.const_labels(
							vec![
								("command".to_string(), command.to_string()),
								("field_id".to_string(), format!("{:?}", field_id)),
							]
							.into_iter()
							.collect(),
						)
				}),
			),
		}
	}

	pub(crate) fn on_change_object_count(&mut self, name: &String, delta: i64) {
		self.object_count.measurer(name).add(delta);
	}

	pub(crate) fn on_change_member_count(&mut self, name: &String, delta: i64) {
		self.member_count.measurer(name).add(delta)
	}

	pub(crate) fn on_create_room(&mut self, name: &String) {
		self.room_count.measurer(name).inc();
	}

	pub(crate) fn on_output_commands(&mut self, template: &HeaplessStatisticString, commands: &[CommandWithChannelType]) {
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::S2CWithCreator(ref c) = c.command {
				let command = &c.command;
				let key = (command.get_field_type(), command.get_field_id(), template.clone());
				self.outcome_command_count.measurer(&key).inc();
			}
		});
	}

	pub(crate) fn on_input_commands(&mut self, template: &String, commands: &[CommandWithChannel]) {
		let template = HeaplessStatisticString::from(template.as_str());
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::C2S(ref c) = c.both_direction_command {
				let key = (c.get_field_type(), c.get_field_id(), template.clone());
				self.income_command_count.measurer(&key).inc()
			}
		});
	}

	pub(crate) fn on_execute_command(&mut self, field_id: Option<FieldId>, command: &C2SCommand, duration: Duration) {
		let name = command.as_ref();
		let key = (HeaplessStatisticString::from(name), field_id);
		self.execution_time.measurer(&key).observe(duration.as_secs_f64());
	}

	fn measurer_label_factory(
		name: &str,
		help: &str,
	) -> Box<LabelFactoryFactory<(Option<FieldType>, Option<FieldId>, heapless::String<50>), Opts>> {
		let name = name.to_string();
		let help = help.to_string();
		Box::new(move |(t, id, template)| {
			Opts::new(name.as_str(), help.as_str()).const_labels(
				vec![
					(
						"field_type".to_string(),
						t.map(|f| Into::<&str>::into(f).into())
							.unwrap_or_else(|| "unknown".to_string()),
					),
					(
						"field_id".to_string(),
						id.map(|f| format!("{}", f)).unwrap_or_else(|| "unknown".to_string()),
					),
					("template".to_string(), template.to_string()),
				]
				.into_iter()
				.collect(),
			)
		})
	}
}
