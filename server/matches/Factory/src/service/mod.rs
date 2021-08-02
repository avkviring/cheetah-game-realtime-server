use crate::proto::matches::relay::types as relay;
use std::collections::HashMap;
use std::path::Path;

pub mod converter;
pub mod grpc;
pub mod room;

pub struct Service {
    registry: grpc::RegistryClient,
    templates: HashMap<String, relay::RoomTemplate>,
}

impl Service {
    pub fn new(registry: grpc::RegistryClient, path: &Path) -> Result<Self, String> {
        let templates = room::Room::load_dir(path)?;
        Ok(Self {
            registry,
            templates,
        })
    }

    pub fn template(&self, template: &str) -> Option<relay::RoomTemplate> {
        self.templates.get(template).cloned()
    }
}

#[cfg(test)]
mod test {
    use crate::service::{grpc, room, Service};
    use std::path::PathBuf;

    #[test]
    pub fn should_factory_service_load_templates() {
        let tmp = tempfile::TempDir::new().unwrap();
        let room = room::Room::default();

        let room_str = serde_yaml::to_string(&room).unwrap();

        write_file(tmp.path().join("kungur.yaml"), &room_str).unwrap();
        write_file(tmp.path().join("ctf/gubaha.yaml"), &room_str).unwrap();

        let registry = grpc::RegistryClient::new("not-used").unwrap();
        let service = Service::new(registry, tmp.path()).unwrap();
        println!("{:?}", service.templates.keys());

        assert!(service.templates.get("/kungur").is_some());
        assert!(service.templates.get("/ctf/gubaha").is_some());
    }

    #[test]
    pub fn should_fail_if_wrong_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let registry = grpc::RegistryClient::new("not-used").unwrap();
        write_file(tmp.path().join("kungur.yaml"), &"not-yaml-file".to_string()).unwrap();
        assert!(Service::new(registry, tmp.path()).is_err());
    }

    pub fn write_file(out: PathBuf, contents: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(out.parent().unwrap())?;
        std::fs::write(out, contents)
    }
}
