use std::collections::HashMap;
use std::convert::TryFrom;

use crate::proto::matches::relay::internal as relay;
use crate::service::configurations::Configurations;
use crate::service::grpc::registry_client::RegistryClient;
use crate::service::resolver::error;

pub mod configurations;
pub mod grpc;
pub mod resolver;

pub struct FactoryService {
	registry: RegistryClient,
	templates: HashMap<String, relay::RoomTemplate>,
}

impl FactoryService {
	pub fn new(registry: RegistryClient, configurations: &Configurations) -> Result<Self, error::Error> {
		let templates = TryFrom::try_from(configurations)?;
		Ok(Self { registry, templates })
	}

	pub fn template(&self, template: &str) -> Option<relay::RoomTemplate> {
		self.templates.get(template).cloned()
	}
}
