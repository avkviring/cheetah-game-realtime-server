use cheetah_libraries_microservice::tonic::{Request, Response};

use crate::proto::matches::factory::admin;
use crate::proto::matches::realtime::shared;
use crate::service::configuration::yaml::structures::FieldType;
use crate::service::configuration::yaml::YamlConfigurations;

pub struct ConfigurationsService {
	templates: Vec<admin::TemplateInfo>,
	fields: Vec<admin::FieldInfo>,
}

impl ConfigurationsService {
	pub fn new(config: &YamlConfigurations) -> ConfigurationsService {
		ConfigurationsService {
			templates: config
				.templates
				.iter()
				.map(|(name, template)| admin::TemplateInfo {
					id: template.id,
					name: name.to_string(),
				})
				.collect(),
			fields: config
				.fields
				.iter()
				.map(|(name, field)| admin::FieldInfo {
					id: field.id as u32,
					r#type: to_admin_field_type(&field.r#type),
					name: name.to_string(),
				})
				.collect(),
		}
	}
}

fn to_admin_field_type(field_type: &FieldType) -> i32 {
	match field_type {
		FieldType::Long => shared::FieldType::Long as i32,
		FieldType::Double => shared::FieldType::Double as i32,
		FieldType::Struct => shared::FieldType::Structure as i32,
		FieldType::Event => shared::FieldType::Event as i32,
	}
}

#[tonic::async_trait]
impl admin::configurations_server::Configurations for ConfigurationsService {
	async fn get_item_names(&self, _: Request<admin::GetItemsNamesRequest>) -> Result<Response<admin::GetItemsNamesResponse>, tonic::Status> {
		Result::Ok(Response::new(admin::GetItemsNamesResponse {
			templates: self.templates.clone(),
			fields: self.fields.clone(),
		}))
	}
}

#[cfg(test)]
pub mod tests {
	use crate::proto::matches::factory::admin;
	use crate::proto::matches::realtime::shared;
	use crate::service::admin::ConfigurationsService;
	use crate::service::configuration::yaml::structures::{Field, FieldType, Template};
	use crate::service::configuration::yaml::YamlConfigurations;

	#[test]
	fn should_convert_templates() {
		let conf = YamlConfigurations {
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
		assert_eq!(
			*service.templates.first().unwrap(),
			admin::TemplateInfo {
				id: 10,
				name: "tank".to_string()
			}
		);
	}

	#[test]
	fn should_convert_fields() {
		let conf = YamlConfigurations {
			groups: Default::default(),
			fields: vec![(
				"score".to_string(),
				Field {
					name: None,
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
		assert_eq!(
			*service.fields.first().unwrap(),
			admin::FieldInfo {
				id: 10,
				r#type: shared::FieldType::Long as i32,
				name: "score".to_string()
			}
		);
	}
}
