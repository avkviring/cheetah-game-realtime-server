use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector;
use kube::core::params::PostParams;
use kube::core::{CustomResourceExt, Resource};
use kube::{Api, Client};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

///
///
/// Описание CRD ресурса GameServerAllocationSpec
/// https://agones.dev/site/docs/reference/agones_crd_api_reference/#allocation.agones.dev/v1.GameServerAllocation
///
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, Validate, JsonSchema, Default)]
#[kube(status = "GameServerAllocationStatus")]
#[kube(group = "allocation.agones.dev", version = "v1", kind = "GameServerAllocation", namespaced)]
pub struct GameServerAllocationSpec {
	selectors: Vec<GameServerSelector>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct GameServerSelector {}
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct GameServerAllocationStatus {
	pub state: GameServerAllocationState,
	#[serde(rename = "gameServerName")]
	pub game_server_name: String,
	pub ports: Vec<GameServerStatusPort>,
	pub address: String,
	#[serde(rename = "nodeName")]
	pub node_name: String,
}
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub enum GameServerAllocationState {
	Allocated,
	UnAllocated,
}
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct GameServerStatusPort {
	pub name: String,
	pub port: u32,
}
