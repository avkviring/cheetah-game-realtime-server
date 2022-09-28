use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::Arc;

use thiserror::Error;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::{RoomsServerManager, RoomsServerManagerError, Server};

///
/// Паттерн Создатель для игрового сервера
/// - если адреса для udp/grpc не заданы - то в качестве адреса выбирается 127.0.0.1, в качестве
/// порта - свободный порт
///
pub struct ServerBuilder {
	game_bind_addr: SocketAddr,
	admin_grpc_bind_addr: SocketAddr,
	internal_grpc_bind_addr: SocketAddr,
	is_agones_enabled: bool,
}

impl Default for ServerBuilder {
	fn default() -> Self {
		Self {
			game_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			admin_grpc_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			internal_grpc_bind_addr: SocketAddr::from_str("127.0.0.1:0").unwrap(),
			is_agones_enabled: false,
		}
	}
}

#[derive(Debug, Error)]
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
	pub fn set_game_address(mut self, addr: SocketAddr) -> Self {
		self.game_bind_addr = addr;
		self
	}

	pub fn set_admin_grpc_address(mut self, addr: SocketAddr) -> Self {
		self.admin_grpc_bind_addr = addr;
		self
	}

	pub fn set_internal_grpc_address(mut self, addr: SocketAddr) -> Self {
		self.internal_grpc_bind_addr = addr;
		self
	}

	pub fn enable_agones(mut self) -> Self {
		self.is_agones_enabled = true;
		self
	}

	pub async fn build(self) -> Result<Server, ServerBuilderError> {
		let game_socket = UdpSocket::bind(self.game_bind_addr).map_err(|e| ServerBuilderError::ErrorBindUdpSocket(e))?;
		let game_socket_addr = game_socket
			.local_addr()
			.map_err(|e| ServerBuilderError::ErrorGetLocalAddrFromUdpSocket(e))?;
		let server_manager = RoomsServerManager::new(game_socket).map_err(|e| ServerBuilderError::RoomsServerManager(e))?;
		let manager = Arc::new(Mutex::new(server_manager));
		let internal_grpc_bind_listener = TcpListener::bind(self.internal_grpc_bind_addr)
			.await
			.map_err(|e| ServerBuilderError::ErrorOpenGrpcSocket(e))?;
		let admin_grpc_bind_listener = TcpListener::bind(self.admin_grpc_bind_addr)
			.await
			.map_err(|e| ServerBuilderError::ErrorOpenGrpcSocket(e))?;

		Ok(Server {
			game_socket_addr,
			internal_grpc_tcp_listener: internal_grpc_bind_listener,
			admin_grpc_tcp_listener: admin_grpc_bind_listener,
			is_agones_enabled: self.is_agones_enabled,
			manager,
		})
	}
}
