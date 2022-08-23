use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

use prometheus::{IntCounter, Opts};
use tonic::Status;

use cheetah_libraries_microservice::trace::Trace;

use crate::proto::matches::factory::internal::CreateMatchResponse;
use crate::proto::matches::realtime::internal as realtime;
use crate::proto::matches::realtime::internal::realtime_client::RealtimeClient;
use crate::service::configuration::converter::error;
use crate::service::configuration::yaml::YamlConfigurations;
use crate::service::grpc::registry_client::RegistryClient;

pub mod admin;
pub mod configuration;
pub mod grpc;

pub struct FactoryService {
	registry: RegistryClient,
	templates: HashMap<String, realtime::RoomTemplate>,
	prometheus_counters: Mutex<HashMap<String, IntCounter>>,
}

impl FactoryService {
	pub fn new(
		registry: RegistryClient,
		configurations: &YamlConfigurations,
	) -> Result<Self, error::Error> {
		let templates = TryFrom::try_from(configurations)?;
		Ok(Self {
			registry,
			templates,
			prometheus_counters: Mutex::new(Default::default()),
		})
	}

	async fn do_create_match(&self, template_name: String) -> Result<CreateMatchResponse, Status> {
		self.prometheus_increment_create_match_counter(template_name.as_str());

		// получаем шаблон
		let room_template = self
			.template(&template_name)
			.ok_or(())
			.trace_err(format!("Template {} not found", template_name))
			.map_err(Status::internal)?;

		// ищем свободный relay сервер
		let addrs = self
			.registry
			.find_free_relay()
			.await
			.trace_err("Find free relay server")
			.map_err(Status::internal)?;

		let relay_grpc_addr = addrs.grpc_internal.as_ref().unwrap();
		let relay_addr = cheetah_libraries_microservice::make_internal_srv_uri(
			&relay_grpc_addr.host,
			relay_grpc_addr.port as u16,
		);
		// создаем матч на relay сервере
		let mut relay_client = RealtimeClient::connect(relay_addr.clone())
			.await
			.trace_err("Create RelayClient connection to {:?}")
			.map_err(Status::internal)?;

		// создаем комнату
		Ok(CreateMatchResponse {
			id: relay_client
				.create_room(room_template.clone())
				.await
				.trace_err(format!(
					"Create Room with template {}",
					room_template.template_name
				))
				.map_err(Status::internal)?
				.into_inner()
				.room_id,
			addrs: Some(addrs),
		})
	}

	pub fn template(&self, template: &str) -> Option<realtime::RoomTemplate> {
		self.templates.get(template).cloned()
	}

	fn prometheus_increment_create_match_counter(&self, template_name: &str) {
		let mut lock_counters = self.prometheus_counters.lock();
		let counters = lock_counters.as_mut().unwrap();
		let counter = counters.entry(template_name.into()).or_insert_with(|| {
			let opts = Opts::new("create_match_counter", "New match counter").const_labels(
				[("template".into(), template_name.into())]
					.into_iter()
					.collect(),
			);
			let counter = IntCounter::with_opts(opts).unwrap();
			prometheus::default_registry()
				.register(Box::new(counter.clone()))
				.unwrap();
			counter
		});
		counter.inc();
	}
}
