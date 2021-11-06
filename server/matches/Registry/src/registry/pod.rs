use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("GameServer has empty status")]
	GameServerEmptyStatus,
	#[error("GameServer ip not set")]
	GameServerIpNotSet,
}

///
/// Получить IP адрес для pod по его имени
///
pub async fn get_pod_ip(name: &str) -> Result<String, Box<dyn std::error::Error>> {
	let client = Client::try_default().await?;
	let crd: Api<Pod> = Api::default_namespaced(client.clone());
	let pod = crd.get(name).await?;
	match pod.status {
		None => Result::Err(Box::new(Error::GameServerEmptyStatus)),
		Some(status) => match status.pod_ip {
			None => Result::Err(Box::new(Error::GameServerIpNotSet)),
			Some(ip) => Result::Ok(ip),
		},
	}
}
