use crate::proto::matches::registry::internal::{Addr, RelayAddrs};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddrsError {
	#[error("malformed RelayAddrs")]
	MalformedAddrs,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Addrs {
	pub(crate) game: SocketAddr,
	pub(crate) grpc_internal: SocketAddr,
}

impl TryFrom<Option<RelayAddrs>> for Addrs {
	type Error = AddrsError;

	fn try_from(val: Option<RelayAddrs>) -> Result<Self, Self::Error> {
		let addrs = val.ok_or(AddrsError::MalformedAddrs)?;
		let game = addrs.game.ok_or(AddrsError::MalformedAddrs)?.try_into()?;
		let grpc_internal = addrs.grpc_internal.ok_or(AddrsError::MalformedAddrs)?.try_into()?;

		Ok(Addrs { game, grpc_internal })
	}
}

impl From<Addrs> for RelayAddrs {
	fn from(a: Addrs) -> Self {
		Self {
			game: Some(a.game.into()),
			grpc_internal: Some(a.grpc_internal.into()),
		}
	}
}

impl TryFrom<Addr> for SocketAddr {
	type Error = AddrsError;

	#[allow(clippy::map_err_ignore)]
	fn try_from(a: Addr) -> Result<Self, Self::Error> {
		let ip = IpAddr::from_str(&a.host).map_err(|_| AddrsError::MalformedAddrs)?;
		Ok(SocketAddr::new(ip, a.port.try_into().map_err(|_| AddrsError::MalformedAddrs)?))
	}
}

impl From<SocketAddr> for Addr {
	fn from(a: SocketAddr) -> Self {
		Self {
			host: a.ip().to_string(),
			port: u32::from(a.port()),
		}
	}
}
