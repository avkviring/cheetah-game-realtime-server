use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::EmbeddedServerWrapper;

pub mod logs;
pub mod member;
pub mod room;
pub mod server;

lazy_static! {
	static ref REGISTRY: std::sync::Mutex<Registry> = std::sync::Mutex::new(Default::default());
}

pub type ServerId = u64;

#[derive(Default)]
struct Registry {
	pub next_server_id: ServerId,
	pub servers: HashMap<u64, EmbeddedServerWrapper>,
}
