use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use cheetah_protocol::coniguration::ProtocolConfiguration;

use crate::{RoomsServerManagerError, Server, ServerManager};

///
/// Паттерн Создатель для игрового сервера
/// - если адреса для udp/grpc не заданы - то в качестве адреса выбирается 127.0.0.1, в качестве
/// порта - свободный порт
///
pub struct ServerBuilder {
	game_bind_addr: SocketAddr,
	debug_rest_service_bind_address: SocketAddr,
	internal_grpc_service_bind_address: SocketAddr,
	internal_webgrpc_service_bind_address: SocketAddr,
	is_agones_enabled: bool,
	protocol_configuration: ProtocolConfiguration,
}

impl Default for ServerBuilder {
	fn default() -> Self {
		Self {
			game_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			debug_rest_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_grpc_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_webgrpc_service_bind_address: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			is_agones_enabled: false,
			protocol_configuration: ProtocolConfiguration {
				disconnect_timeout: Duration::from_secs(180),
			},
		}
	}
}

#[derive(Error, Debug)]
pub enum ServerBuilderError {
	#[error("RoomsServerManager {0}")]
	RoomsServerManager(RoomsServerManagerError),
	#[error("ErrorOpenGrpcSocket {0}")]
	ErrorOpenGrpcSocket(io::Error),
	#[error("ErrorBindUdpSocket {0}")]
	ErrorBindUdpSocket(io::Error),
	#[error("ErrorGetLocalAddrFromUdpSocket {0}")]
	ErrorGetLocalAddrFromUdpSocket(io::Error),
}

impl ServerBuilder {
	#[must_use]
	pub fn set_games_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.game_bind_addr = addr;
		self
	}

	#[must_use]
	pub fn set_debug_rest_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.debug_rest_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn set_internal_grpc_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.internal_grpc_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn set_internal_webgrpc_service_bind_address(mut self, addr: SocketAddr) -> Self {
		self.internal_webgrpc_service_bind_address = addr;
		self
	}

	#[must_use]
	pub fn enable_agones(mut self) -> Self {
		self.is_agones_enabled = true;
		self
	}

	#[must_use]
	pub fn set_disconnect_duration(mut self, disconnect_timeout: Duration) -> Self {
		self.protocol_configuration.disconnect_timeout = disconnect_timeout;
		self
	}

	pub async fn build(self) -> Result<Server, ServerBuilderError> {
		let game_socket = UdpSocket::bind(self.game_bind_addr).map_err(ServerBuilderError::ErrorBindUdpSocket)?;
		let game_socket_addr = game_socket.local_addr().map_err(ServerBuilderError::ErrorGetLocalAddrFromUdpSocket)?;
		let server_manager = ServerManager::new(game_socket, self.protocol_configuration).map_err(ServerBuilderError::RoomsServerManager)?;
		let manager = Arc::new(Mutex::new(server_manager));

		let internal_grpc_listener = TcpListener::bind(self.internal_grpc_service_bind_address).await.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;
		let internal_webgrpc_listener = TcpListener::bind(self.internal_webgrpc_service_bind_address).await.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;
		let debug_rest_service_listener = TcpListener::bind(self.debug_rest_service_bind_address).await.map_err(ServerBuilderError::ErrorOpenGrpcSocket)?;

		Ok(Server {
			game_socket_addr,
			internal_webgrpc_listener,
			internal_grpc_listener,
			debug_rest_service_listener,
			is_agones_enabled: self.is_agones_enabled,
			manager,
		})
	}
}
