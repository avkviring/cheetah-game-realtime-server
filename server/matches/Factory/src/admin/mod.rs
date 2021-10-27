use std::collections::HashMap;

use cheetah_microservice::tonic::{Request, Response, Status};

use crate::proto::matches::factory::admin;
use crate::service::configurations::Configurations;

pub struct ConfigurationsService {
	templates: HashMap<u32, String>,
	fields: HashMap<u32, String>,
}

impl ConfigurationsService {
	pub fn new(config: &Configurations) -> ConfigurationsService {
		ConfigurationsService {
			templates: config
				.templates
				.iter()
				.map(|(name, template)| (template.id, name.to_owned()))
				.collect(),
			fields: config
				.fields
				.iter()
				.map(|(name, field)| (field.id as u32, name.to_owned()))
				.collect(),
		}
	}
}

#[tonic::async_trait]
impl admin::configurations_server::Configurations for ConfigurationsService {
	async fn get_item_names(
		&self,
		request: Request<admin::GetItemsNamesRequest>,
	) -> Result<Response<admin::GetItemsNamesResponse>, tonic::Status> {
		Result::Ok(Response::new(admin::GetItemsNamesResponse {
			templates: self.templates.clone(),
			fields: self.fields.clone(),
		}))
	}
}

#[cfg(test)]
pub mod tests {
	use crate::admin::ConfigurationsService;
	use crate::service::configurations::structures::{Field, FieldType, Template};
	use crate::service::configurations::Configurations;

	#[test]
	fn should_convert_templates() {
		let conf = Configurations {
			groups: Default::default(),
			fields: Default::default(),
			templates: vec![(
				"tank".to_string(),
				Template {
					id: 10,
					permissions: Default::default(),
				},
			)]
			.into_iter()
			.collect(),
			rooms: Default::default(),
		};
		let service: ConfigurationsService = ConfigurationsService::new(&conf);
		assert_eq!(service.templates.get(&10).unwrap(), "tank");
	}

	#[test]
	fn should_convert_fields() {
		let conf = Configurations {
			groups: Default::default(),
			fields: vec![(
				"score".to_string(),
				Field {
					id: 10,
					r#type: FieldType::Long,
				},
			)]
			.into_iter()
			.collect(),
			templates: Default::default(),
			rooms: Default::default(),
		};
		let service: ConfigurationsService = ConfigurationsService::new(&conf);
		assert_eq!(service.fields.get(&10).unwrap(), "score");
	}
}
