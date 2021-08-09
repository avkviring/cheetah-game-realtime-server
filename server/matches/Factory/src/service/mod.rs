use crate::proto::matches::relay::types as relay;
use std::collections::HashMap;
use std::path::Path;

pub mod grpc;
pub mod room;

//pub mod converter;
//pub mod old_room;

pub struct Service {
    registry: grpc::RegistryClient,
    templates: HashMap<String, relay::RoomTemplate>,
}

impl Service {
    pub fn new(registry: grpc::RegistryClient, path: &Path) -> Result<Self, String> {
        let templates = room::load_dir(path)?;
        if templates.is_empty() {
            return Err("no templates".to_string());
        }
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
    use std::path::Path;

    #[test]
    pub fn should_factory_service_load_templates() {
        let tmp = tempfile::TempDir::new().unwrap();

        let groups_path = Path::new("groups.yaml");
        let groups = room::Groups::default();
        let contents = serde_yaml::to_string(&groups).unwrap();
        write_file(tmp.path().join(&groups_path), &contents);
        write_file(tmp.path().join("ctf").join(&groups_path), &contents);

        let room = room::Room {
            groups: groups_path.into(),
            ..room::Room::default()
        };
        let room_str = serde_yaml::to_string(&room).unwrap();

        write_file(tmp.path().join("kungur.yaml"), &room_str);
        write_file(tmp.path().join("ctf/gubaha.yaml"), &room_str);

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
        write_file(tmp.path().join("kungur.yaml"), &"not-yaml-file".to_string());
        assert!(Service::new(registry, tmp.path()).is_err());
    }

    pub fn write_file(path: impl AsRef<Path>, contents: &str) {
        let path = path.as_ref();
        println!("write_file: {:?}", path.display());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
}
