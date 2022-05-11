use std::time::Duration;

use prometheus::{Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry};

use cheetah_libraries_microservice::prometheus::measurer::create_and_register_measurer;
use cheetah_libraries_microservice::prometheus::measurers_by_label::{
	HistogramMeasurersByLabel, IntCounterMeasurersByLabel, LabelFactoryFactory, MeasurersByLabel,
};
use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};

pub type MeasureStringId = heapless::String<50>;
type RoomTemplateString = heapless::String<50>;

///
/// Измерение параметров сервера - сохранение в prometheus
///
pub struct Measurers {
	///
	/// Количество комнат на сервере
	///
	room_count: MeasurersByLabel<String, IntGauge, Opts>,
	///
	/// Количество пользователей на сервере (во всех комнатах)
	///
	member_count: MeasurersByLabel<String, IntGauge, Opts>,
	///
	/// Количество объектов на сервере (во всех комнатах)
	///
	object_count: MeasurersByLabel<String, IntGauge, Opts>,
	///
	/// Количество входящих команд
	///
	income_command_count: IntCounterMeasurersByLabel<(Option<FieldType>, Option<FieldId>, RoomTemplateString)>,
	///
	/// Количество исходящих команд
	///
	outcome_command_count: IntCounterMeasurersByLabel<(Option<FieldType>, Option<FieldId>, RoomTemplateString)>,
	///
	/// Время выполнение команд
	///
	input_command_execution_time: HistogramMeasurersByLabel<(MeasureStringId, Option<FieldId>)>,
	///
	/// Размер входящего фрейма
	///
	input_frame_size: Histogram,
	///
	/// Время выполнения фрейма
	input_frame_execution_time: Histogram,
	///
	/// Время выполнения серверного цикла
	///
	server_cycle_execution_time: Histogram,
}

impl Measurers {
	pub fn new(registry: &Registry) -> Self {
		Self {
			room_count: Self::create_room_count_measurers(registry),
			member_count: Self::create_member_count_measurers(registry),
			object_count: Self::create_object_count_measurers(registry),
			income_command_count: Self::create_income_command_count_measurers(registry),
			outcome_command_count: Self::create_outcome_command_count_measurers(registry),
			input_command_execution_time: Self::create_execution_command_time_measurers(registry),
			input_frame_size: Self::create_input_frame_size(registry),
			input_frame_execution_time: Self::create_input_frame_time(registry),
			server_cycle_execution_time: Self::create_server_cycle_execution_time(registry),
		}
	}

	fn create_server_cycle_execution_time(registry: &Registry) -> Histogram {
		create_and_register_measurer(
			registry,
			HistogramOpts::new("server_cycle_execution_time", "Server cycle execution time").buckets(vec![
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

	fn create_input_frame_time(registry: &Registry) -> Histogram {
		create_and_register_measurer(
			registry,
			HistogramOpts::new("input_frame_execution_time", "Input frame execution time").buckets(vec![
				Duration::from_micros(10).as_secs_f64(),
				Duration::from_micros(50).as_secs_f64(),
				Duration::from_micros(100).as_secs_f64(),
				Duration::from_micros(500).as_secs_f64(),
				Duration::from_micros(900).as_secs_f64(),
				Duration::from_millis(1).as_secs_f64(),
				Duration::from_millis(5).as_secs_f64(),
				Duration::from_millis(10).as_secs_f64(),
				Duration::from_millis(50).as_secs_f64(),
			]),
		)
	}

	fn create_input_frame_size(registry: &Registry) -> Histogram {
		create_and_register_measurer(
			registry,
			HistogramOpts::new("input_frame_size", "Input frame size").buckets(vec![100.0, 200.0, 400.0, 800.0, 1200.0, 1500.0]),
		)
	}

	fn create_execution_command_time_measurers(
		registry: &Registry,
	) -> MeasurersByLabel<(MeasureStringId, Option<FieldId>), Histogram, HistogramOpts> {
		MeasurersByLabel::new(
			registry,
			Box::new(|(command, field_id)| {
				HistogramOpts::new("command_execution_time", "command execution time")
					.buckets(vec![
						Duration::from_micros(10).as_secs_f64(),
						Duration::from_micros(50).as_secs_f64(),
						Duration::from_micros(100).as_secs_f64(),
						Duration::from_micros(500).as_secs_f64(),
						Duration::from_micros(900).as_secs_f64(),
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
		)
	}

	fn create_outcome_command_count_measurers(
		registry: &Registry,
	) -> MeasurersByLabel<(Option<FieldType>, Option<FieldId>, MeasureStringId), IntCounter, Opts> {
		MeasurersByLabel::new(
			registry,
			Self::network_command_measurer_label_factory("outcome_command_counter", "Outcome command counter"),
		)
	}

	fn create_income_command_count_measurers(
		registry: &Registry,
	) -> MeasurersByLabel<(Option<FieldType>, Option<FieldId>, MeasureStringId), IntCounter, Opts> {
		MeasurersByLabel::new(
			registry,
			Self::network_command_measurer_label_factory("income_command_counter", "Income command counter"),
		)
	}

	fn create_object_count_measurers(registry: &Registry) -> MeasurersByLabel<String, IntGauge, Opts> {
		MeasurersByLabel::new(
			registry,
			Box::new(|template| {
				Opts::new("object_count", "object count")
					.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
			}),
		)
	}

	fn create_member_count_measurers(registry: &Registry) -> MeasurersByLabel<String, IntGauge, Opts> {
		MeasurersByLabel::new(
			registry,
			Box::new(|template| {
				Opts::new("member_count", "member count")
					.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
			}),
		)
	}

	fn create_room_count_measurers(registry: &Registry) -> MeasurersByLabel<String, IntGauge, Opts> {
		MeasurersByLabel::new(
			registry,
			Box::new(|template| {
				Opts::new("room_count", "room count")
					.const_labels(vec![("template".to_string(), template.clone())].into_iter().collect())
			}),
		)
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

	pub(crate) fn on_output_commands(&mut self, template: &MeasureStringId, commands: &[CommandWithChannelType]) {
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::S2CWithCreator(ref c) = c.command {
				let command = &c.command;
				let key = (command.get_field_type(), command.get_field_id(), template.clone());
				self.outcome_command_count.measurer(&key).inc();
			}
		});
	}

	pub(crate) fn on_input_commands(&mut self, template: &String, commands: &[CommandWithChannel]) {
		let template = MeasureStringId::from(template.as_str());
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::C2S(ref c) = c.both_direction_command {
				let key = (c.get_field_type(), c.get_field_id(), template.clone());
				self.income_command_count.measurer(&key).inc()
			}
		});
	}

	pub(crate) fn on_execute_command(&mut self, field_id: Option<FieldId>, command: &C2SCommand, duration: Duration) {
		let name = command.as_ref();
		let key = (MeasureStringId::from(name), field_id);
		self.input_command_execution_time
			.measurer(&key)
			.observe(duration.as_secs_f64());
	}

	pub(crate) fn on_income_frame(&mut self, size: usize, duration: Duration) {
		self.input_frame_size.observe(size as f64);
		self.input_frame_execution_time.observe(duration.as_secs_f64());
	}

	pub(crate) fn on_server_cycle(&mut self, duration: Duration) {
		self.server_cycle_execution_time.observe(duration.as_secs_f64())
	}

	fn network_command_measurer_label_factory(
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
