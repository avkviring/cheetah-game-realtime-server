use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use tonic::transport::Uri;

use crate::proto::matches::relay::types as relay_types;
use crate::service::yaml::{RoomTemplate, RoomTemplateError};

pub mod converter;
pub mod grpc;
pub mod yaml;

pub struct FactoryService {
    registry_service: Uri,
    templates: HashMap<String, relay_types::RoomTemplate>,
}

#[derive(Debug)]
pub struct LoadRoomTemplateError {
    pub name: String,
    pub message: String,
}

impl LoadRoomTemplateError {
    fn convert(source: RoomTemplateError, template_name: &str) -> Self {
        let message = match source {
            RoomTemplateError::YamlParserError(e) => match e.location() {
                None => "Wrong file format".to_string(),
                Some(location) => {
                    format!(
                        "Wrong file format in position {:?}:{:?}",
                        location.line(),
                        location.column()
                    )
                }
            },
            RoomTemplateError::YamlContainsUnmappingFields(e) => {
                format!("{:?}", e)
            }
        };
        LoadRoomTemplateError {
            name: template_name.to_owned(),
            message,
        }
    }
}

impl FactoryService {
    pub fn new(registry_service: Uri, path: &Path) -> Result<Self, LoadRoomTemplateError> {
        let templates = FactoryService::load_templates(path, "")?;
        Result::Ok(FactoryService {
            registry_service,
            templates,
        })
    }
    pub fn get_template(&self, template: &str) -> Option<relay_types::RoomTemplate> {
        self.templates.get(template).cloned()
    }

    fn load_templates(
        root: &Path,
        prefix: &str,
    ) -> Result<HashMap<String, relay_types::RoomTemplate>, LoadRoomTemplateError> {
        let mut result = HashMap::new();

        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            // пропускаем служебные каталоги при монтировании ConfigMap в kubernetes
            if path.to_str().unwrap().contains("..") {
                continue;
            }
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if path.is_dir() {
                let child_prefix = format!("{}/{}", prefix, file_name);
                result.extend(FactoryService::load_templates(
                    &path,
                    child_prefix.as_str(),
                )?);
            } else if file_name.ends_with("yaml") || file_name.ends_with("yml") {
                let template_name = format!(
                    "{}/{}",
                    prefix,
                    file_name.replace(".yml", "").replace(".yaml", "")
                );
                let template = FactoryService::load_template(&path, template_name.as_str())?;
                result.insert(template_name, template.into());
            }
        }

        Result::Ok(result)
    }

    fn load_template(
        path: &Path,
        template_name: &str,
    ) -> Result<RoomTemplate, LoadRoomTemplateError> {
        log::info!("load room {:?}", template_name);
        let mut file = std::fs::File::open(&path).unwrap();
        let mut content = String::default();
        file.read_to_string(&mut content).unwrap();
        let template = RoomTemplate::new_from_yaml(content.as_str())
            .map_err(|e| LoadRoomTemplateError::convert(e, template_name))?;

        Result::Ok(template)
    }
}
#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::str::FromStr;

    use tonic::transport::Uri;

    use crate::service::yaml;
    use crate::service::FactoryService;

    #[test]
    pub fn should_factory_service_load_templates() {
        let templates_directory = tempfile::TempDir::new().unwrap();
        let room_template = yaml::RoomTemplate {
            objects: vec![],
            permissions: Default::default(),
            unmapping: Default::default(),
        };

        let room_template_as_string = serde_yaml::to_string(&room_template).unwrap();

        write_file(
            templates_directory.path().join("kungur.yaml"),
            &room_template_as_string,
        );

        write_file(
            templates_directory.path().join("ctf/gubaha.yaml"),
            &room_template_as_string,
        );

        let service = FactoryService::new(
            Uri::from_str("not-used").unwrap(),
            templates_directory.path(),
        )
        .unwrap();
        println!("{:?}", service.templates.keys());
        assert!(service.templates.get("/kungur").is_some());
        assert!(service.templates.get("/ctf/gubaha").is_some());
    }

    #[test]
    pub fn should_fail_if_wrong_file() {
        let templates_directory = tempfile::TempDir::new().unwrap();
        write_file(
            templates_directory.path().join("kungur.yaml"),
            &"not-yaml-file".to_string(),
        );
        assert!(FactoryService::new(
            Uri::from_str("not-used").unwrap(),
            templates_directory.path(),
        )
        .is_err());
    }

    pub fn write_file(out: PathBuf, content: &str) {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        let mut room_file = File::create(&out).unwrap();
        room_file.write_all(content.as_bytes()).unwrap();
        room_file.sync_all().unwrap();
    }
}
