use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde_yaml::Location;
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

impl FactoryService {
    pub fn new(registry_service: Uri, path: &Path) -> Result<Self, LoadRoomTemplateError> {
        let templates = load_templates(path, "")?;
        Result::Ok(FactoryService {
            registry_service,
            templates,
        })
    }
    pub fn get_template(&self, template: &String) -> Option<relay_types::RoomTemplate> {
        self.templates.get(template).cloned()
    }
}

#[derive(Debug)]
pub struct LoadRoomTemplateError {
    pub name: String,
    pub message: String,
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
            let child_templates =
                load_templates(&path, format!("{}/{}", prefix, file_name).as_str())?;
            result.extend(child_templates);
        } else {
            if file_name.ends_with("yaml") || file_name.ends_with("yml") {
                let mut file = std::fs::File::open(&path).unwrap();
                log::info!("load room {:?}", path);
                let mut content = String::default();
                file.read_to_string(&mut content).unwrap();
                let template = RoomTemplate::new_from_yaml(content.as_str()).map_err(|e| {
                    let message = match e {
                        RoomTemplateError::YamlParserError(e) => match e.location() {
                            None => {
                                format!("Wrong file format")
                            }
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
                        name: format!("{}/{}", prefix, file_name),
                        message,
                    }
                })?;
                let template_name = file_name.replace(".yml", "").replace(".yaml", "");
                result.insert(format!("{}/{}", prefix, template_name), template.into());
            }
        }
    }

    Result::Ok(result)
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
        assert_eq!(service.templates.get("/kungur").is_some(), true);
        assert_eq!(service.templates.get("/ctf/gubaha").is_some(), true);
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

    pub fn write_file(out: PathBuf, content: &String) {
        std::fs::create_dir_all(out.parent().unwrap()).unwrap();
        let mut room_file = File::create(&out).unwrap();
        room_file.write_all(content.as_bytes()).unwrap();
        room_file.sync_all().unwrap();
    }
}
