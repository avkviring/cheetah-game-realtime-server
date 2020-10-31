use std::ops::Add;
use std::time::{Duration, Instant};

use rand::RngCore;
use rand::rngs::OsRng;

use cheetah_relay_common::commands::hash::{UserPrivateKey, UserPublicKey};
use cheetah_relay_common::udp::channel::Transport;
use cheetah_relay_common::udp::client::UdpClient;
use cheetah_relay_common::udp::protocol::frame::applications::ApplicationCommand;
use cheetah_relay_common::udp::server::UdpServer;

use crate::udp::stub::{AddressStub, ChannelQuality, TransportStub};

pub mod stub;
pub mod protocol;
pub mod network;

