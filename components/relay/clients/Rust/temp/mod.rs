use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;

use stderrlog::Timestamp;

use cheetah_relay::server::Server;
use cheetah_relay_client::do_create_client;
use cheetah_relay_common::room::{RoomId, UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::bind_to_free_socket;

pub mod connect;

