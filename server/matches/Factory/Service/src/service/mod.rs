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
    pub fn new(registry_grpc_service: String, path: &Path) -> Self {
        let templates = load_templates(path);
        FactoryService {
            registry_grpc_service_address: registry_grpc_service,
            templates,
        }
    }
    pub fn get_template(&self, template: &String) -> Option<relay_types::RoomTemplate> {
        self.templates.get(template).cloned()
    }
}

fn load_templates(path: &Path) -> HashMap<String, relay_types::RoomTemplate> {
    fs::read_dir(path)
        .unwrap()
        .map(|res| {
            let res = res.unwrap();
            let mut file = std::fs::File::open(res.path()).unwrap();
            let mut content = String::default();
            file.read_to_string(&mut content).unwrap();
            let yaml_room_template = RoomTemplate::new_from_yaml(content.as_str()).unwrap();
            (
                res.file_name().to_str().unwrap().to_owned(),
                yaml_room_template.into(),
            )
        })
        .collect()
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;
    use std::time::Duration;

    use crate::service::load_templates;

    //#[test]
    pub fn load_templates_test() {
        let templates_directory = tempfile::TempDir::new().unwrap();
        let room_template = r#"
        objects:
         - id: 5
           template: 5
           access_groups: 0
           fields:
            longs:
                10: 1020
        "#;
        {
            let mut room_file = File::create(templates_directory.path().join("room.yaml")).unwrap();
            room_file.write_all(room_template.as_bytes()).unwrap();
            room_file.sync_all().unwrap();
        }
        let templates = load_templates(templates_directory.path());
        let template = templates.get("room").unwrap();
    }
}
