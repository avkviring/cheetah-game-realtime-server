use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use crate::proto::matches::relay::types as relay_types;
use crate::service::yaml::RoomTemplate;

pub mod converter;
pub mod grpc;
pub mod yaml;

pub struct FactoryService {
    pub registry_grpc_service_address: String,
    templates: HashMap<String, relay_types::RoomTemplate>,
}

impl FactoryService {
    pub fn new(registry_grpc_service: &str, path: &Path) -> Self {
        let templates = load_templates(path, "");
        FactoryService {
            registry_grpc_service_address: registry_grpc_service.to_owned(),
            templates,
        }
    }
    pub fn get_template(&self, template: &String) -> Option<relay_types::RoomTemplate> {
        self.templates.get(template).cloned()
    }
}

fn load_templates(root: &Path, prefix: &str) -> HashMap<String, relay_types::RoomTemplate> {
    let mut result = HashMap::new();

    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            result.extend(load_templates(
                &path,
                format!("{}/{}", prefix, file_name).as_str(),
            ));
        } else {
            if file_name.ends_with("yaml") || file_name.ends_with("yml") {
                let mut file = std::fs::File::open(&path).unwrap();
                let mut content = String::default();
                file.read_to_string(&mut content).unwrap();
                let template = RoomTemplate::new_from_yaml(content.as_str()).unwrap();
                let template_name = file_name.replace(".yml", "").replace(".yaml", "");
                result.insert(format!("{}/{}", prefix, template_name), template.into());
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use std::fs::{DirEntry, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::proto::matches::relay::types as grpc;
    use crate::service::yaml;
    use crate::service::{load_templates, FactoryService};

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

        let service = FactoryService::new("not-used", templates_directory.path());
        assert_eq!(service.templates.get("/kungur").is_some(), true);
        assert_eq!(service.templates.get("/ctf/gubaha").is_some(), true);
    }

    fn write_file(out: PathBuf, content: &String) {
        std::fs::create_dir_all(out.parent().unwrap());
        let mut room_file = File::create(&out).unwrap();
        room_file.write_all(content.as_bytes()).unwrap();
        room_file.sync_all().unwrap();
    }
}
