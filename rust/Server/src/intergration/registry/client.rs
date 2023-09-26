use std::collections::HashMap;
use tonic::transport::Uri;
use tonic::{Request, Status};

use crate::intergration::registry::proto::status;
use crate::intergration::registry::proto::status::{Addr, State};

pub struct RegistryClient {
	client: status::status_receiver_client::StatusReceiverClient<tonic::transport::Channel>,
}

impl RegistryClient {}

impl RegistryClient {
	pub async fn new(uri: Uri) -> Result<Self, tonic::transport::Error> {
		let client = status::status_receiver_client::StatusReceiverClient::connect(uri).await?;
		Ok(Self { client })
	}

	pub async fn update_server_status(&self, game: Addr, grpc_internal: Addr, state: State) -> Result<(), Status> {
		let req = Request::new(status::ServerStatus {
			game: Some(game),
			grpc_internal: Some(grpc_internal),
			state: state as i32,
			envs: RegistryClient::get_envs(),
		});

		self.client.clone().set_status(req).await.map(|_| ())
	}

	fn get_envs() -> HashMap<String, String> {
		match std::env::var("REGISTRY_VAR_NAMES") {
			Ok(names) => {
				return names
					.split(",")
					.map(|v| v.trim())
					.map(|v| (v.into(), std::env::var(v)))
					.filter(|(_, v)| v.is_ok())
					.map(|(k, v)| (k, v.unwrap()))
					.collect();
			}
			Err(_) => {}
		}
		Default::default()
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashMap;
	use std::env;

	use crate::intergration::registry::client::RegistryClient;

	#[test]
	pub fn should_make_envs() {
		env::set_var("REGISTRY_VAR_NAMES", "BUILD_ID,VERSION");
		env::set_var("BUILD_ID", "1445");
		env::set_var("VERSION", "7");
		let map = HashMap::<String, String>::from([("BUILD_ID".into(), "1445".into()), ("VERSION".into(), "7".into())]);
		assert_eq!(RegistryClient::get_envs(), map);
	}
}
