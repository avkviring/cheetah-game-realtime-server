use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::EmbeddedServerWrapper;

pub(crate) mod logs;
pub(crate) mod member;
pub(crate) mod room;
pub(crate) mod server;

lazy_static! {
	static ref REGISTRY: std::sync::Mutex<Registry> = std::sync::Mutex::new(Default::default());
}

pub(crate) type ServerId = u64;

#[derive(Default)]
struct Registry {
	pub(crate) next_server_id: ServerId,
	pub(crate) servers: HashMap<u64, EmbeddedServerWrapper>,
}
