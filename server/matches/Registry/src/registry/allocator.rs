
use kube::core::params::PostParams;
use kube::{Api, Client};
use thiserror::Error;

use crate::registry::spec::*;

#[derive(Error, Debug)]
pub enum Error {
	#[error("GameServerAllocation has empty status")]
	EmptyStatus,
}

///
/// Ищем существующий или создаем kubernetes ресурс GameServerAllocation
/// Agones должен выделить сервер для данного ресурса и вернуть нам его адрес в статусе
///
pub async fn allocate_game_server() -> Result<GameServerAllocationStatus, Box<dyn std::error::Error>> {
	let client = Client::try_default().await?;
	let crd: Api<GameServerAllocation> = Api::default_namespaced(client.clone());
	let resource = GameServerAllocation::new("single-server", GameServerAllocationSpec::default());
	let created = crd.create(&PostParams::default(), &resource).await?;
	created
		.status
		.ok_or_else::<Box<dyn std::error::Error>, _>(|| Box::new(Error::EmptyStatus))
}
