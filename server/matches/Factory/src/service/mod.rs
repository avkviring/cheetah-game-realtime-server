use std::collections::HashMap;
use std::convert::TryFrom;

use crate::proto::matches::relay::types as relay;
use crate::service::configurations::Configurations;
use crate::service::resolver::error;

pub mod configurations;
pub mod grpc;
pub mod resolver;

pub struct Service {
	registry: grpc::RegistryClient,
	templates: HashMap<String, relay::RoomTemplate>,
}

impl Service {
	pub fn new(registry: grpc::RegistryClient, configurations: &Configurations) -> Result<Self, error::Error> {
		let templates = TryFrom::try_from(configurations)?;
		Ok(Self { registry, templates })
	}

	pub fn template(&self, template: &str) -> Option<relay::RoomTemplate> {
		self.templates.get(template).cloned()
	}
}

#[cfg(test)]
mod test {
	// use std::path::Path;
	//
	// use crate::service::{grpc, room, Service};

	#[test]
	pub fn should_factory_service_load_templates() {
		todo!()
		// let tmp = tempfile::TempDir::new().unwrap();
		//
		// let groups = room::Config::Groups { groups: Default::default() };
		// write_file(tmp.path().join(&"groups.yaml"), &groups);
		//
		// let room = room::Config::Room(room::Room {
		// 	groups: "/groups".into(),
		// 	..room::Room::default()
		// });
		//
		// write_file(tmp.path().join("kungur.yaml"), &room);
		// write_file(tmp.path().join("ctf/gubaha.yaml"), &room);
		//
		// let registry = grpc::RegistryClient::new("not-used").unwrap();
		// let service = Service::new(registry, tmp.path()).unwrap();
		// println!("{:?}", service.templates.keys());
		//
		// assert!(service.templates.get("/kungur").is_some());
		// assert!(service.templates.get("/ctf/gubaha").is_some());
	}

	// #[test]
	// pub fn should_fail_if_wrong_file() {
	// 	let tmp = tempfile::TempDir::new().unwrap();
	// 	let registry = grpc::RegistryClient::new("not-used").unwrap();
	// 	write_file_str(&tmp.path().join("kungur.yaml"), "not-yaml-file");
	// 	assert!(Service::new(registry, tmp.path()).is_err());
	// }
	//
	// pub fn write_file(path: impl AsRef<Path>, contents: &room::Config) {
	// 	write_file_str(path, &serde_yaml::to_string(contents).unwrap())
	// }
	//
	// pub fn write_file_str(path: impl AsRef<Path>, contents: &str) {
	// 	let path = path.as_ref();
	// 	println!("write_file: {:?}", path.display());
	// 	std::fs::create_dir_all(path.parent().unwrap()).unwrap();
	// 	std::fs::write(path, contents).unwrap();
	// }
}
