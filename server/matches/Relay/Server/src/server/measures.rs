use prometheus::{IntCounter, IntGauge};

use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::constants::FieldId;
use cheetah_matches_relay_common::protocol::commands::output::CommandWithChannelType;
use cheetah_matches_relay_common::protocol::frame::applications::{BothDirectionCommand, CommandWithChannel};
use cheetah_microservice::prometheus::measurers::{LabelFactoryFactory, MeasurersByLabel};

pub type HeaplessStatisticString = heapless::String<50>;

///
/// Измерение параметров сервера - сохранение в prometheus
///
pub struct ServerMeasures {
	measure_room_count: MeasurersByLabel<String, IntGauge>,
	measure_member_count: MeasurersByLabel<String, IntGauge>,
	measure_object_count: MeasurersByLabel<String, IntGauge>,
	measure_income_command_count: MeasurersByLabel<(Option<FieldType>, Option<FieldId>, HeaplessStatisticString), IntCounter>,
	measure_outcome_command_count: MeasurersByLabel<(Option<FieldType>, Option<FieldId>, HeaplessStatisticString), IntCounter>,
}

impl ServerMeasures {
	pub fn new() -> Self {
		Self {
			measure_room_count: MeasurersByLabel::new(
				"room_count",
				"Room by template",
				prometheus::default_registry().clone(),
				Box::new(|template| vec![("template", template.clone())]),
			),
			measure_member_count: MeasurersByLabel::new(
				"member_count",
				"Member count",
				prometheus::default_registry().clone(),
				Box::new(|template| vec![("template", template.clone())]),
			),
			measure_object_count: MeasurersByLabel::new(
				"object_count",
				"Object count",
				prometheus::default_registry().clone(),
				Box::new(|template| vec![("template", template.to_string())]),
			),
			measure_income_command_count: MeasurersByLabel::new(
				"income_command_counter",
				"Income command counter",
				prometheus::default_registry().clone(),
				Self::measurer_label_factory(),
			),
			measure_outcome_command_count: MeasurersByLabel::new(
				"outcome_command_counter",
				"Outcome command counter",
				prometheus::default_registry().clone(),
				Self::measurer_label_factory(),
			),
		}
	}

	pub(crate) fn on_change_object_count(&mut self, name: &String, delta: i64) {
		self.measure_object_count.measurer(name).add(delta);
	}

	pub(crate) fn on_change_member_count(&mut self, name: &String, delta: i64) {
		self.measure_member_count.measurer(name).add(delta)
	}

	pub(crate) fn on_create_room(&mut self, name: &String) {
		self.measure_room_count.measurer(name).inc();
	}

	pub(crate) fn on_output_commands(&mut self, template: &HeaplessStatisticString, commands: &[CommandWithChannelType]) {
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::S2CWithCreator(ref c) = c.command {
				let command = &c.command;
				let key = (command.get_field_type(), command.get_field_id(), template.clone());
				self.measure_outcome_command_count.measurer(&key).inc();
			}
		});
	}

	pub(crate) fn on_input_commands(&mut self, template: &String, commands: &[CommandWithChannel]) {
		let template = HeaplessStatisticString::from(template.as_str());
		commands.iter().for_each(|c| {
			if let BothDirectionCommand::C2S(ref c) = c.both_direction_command {
				let key = (c.get_field_type(), c.get_field_id(), template.clone());
				self.measure_income_command_count.measurer(&key).inc()
			}
		});
	}

	fn measurer_label_factory() -> Box<LabelFactoryFactory<(Option<FieldType>, Option<FieldId>, heapless::String<50>)>> {
		Box::new(|(t, id, template)| {
			vec![
				(
					"field_type",
					t.map(|f| Into::<&str>::into(f).into())
						.unwrap_or_else(|| "unknown".to_string()),
				),
				(
					"field_id",
					id.map(|f| format!("{}", f)).unwrap_or_else(|| "unknown".to_string()),
				),
				("template", template.to_string()),
			]
		})
	}
}
