use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use cheetah_server::builder::ServerBuilder;
use cheetah_server::server::manager1::ServerManager;

mod ffi;

///
/// Обертка для запуска сервера из so/dll.
/// - методы не могут быть async так как они будут вызываться как методы so/dll
///
pub struct EmbeddedServerWrapper {
	runtime: Runtime,
	pub manager: Arc<Mutex<ServerManager>>,
	pub game_socket_addr: SocketAddr,
	pub internal_grpc_socket_addr: SocketAddr,
	pub internal_webgrpc_socket_addr: SocketAddr,
	pub admin_webgrpc_socket_addr: SocketAddr,
}

#[derive(Error, Debug)]
pub enum EmbeddedServerWrapperError {
	#[error("GrpcServicesNotStarted")]
	GrpcServicesNotStarted,
}

impl EmbeddedServerWrapper {
	pub fn run_new_server(internal_grpc_address: SocketAddr, internal_webgrpc_address: SocketAddr, dump_rest_service_address: SocketAddr, game_udp_address: SocketAddr) -> anyhow::Result<Self> {
		let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_io().enable_time().build()?;

		let server = runtime.block_on(async move {
			ServerBuilder::default()
				.set_internal_grpc_service_bind_address(internal_grpc_address)
				.set_internal_webgrpc_service_bind_address(internal_webgrpc_address)
				.set_debug_rest_service_bind_address(dump_rest_service_address)
				.set_games_service_bind_address(game_udp_address)
				.build()
				.await
		})?;
		let manager = Arc::clone(&server.manager);
		let game_socket_addr = server.game_socket_addr;
		let internal_grpc_socket_addr = server.internal_grpc_listener.local_addr()?;
		let admin_webgrpc_socket_addr = server.debug_rest_service_listener.local_addr()?;
		let internal_webgrpc_socket_addr = server.internal_webgrpc_listener.local_addr()?;
		runtime.spawn(async move {
			server.run().await;
		});

		Self::assert_open_port(internal_grpc_socket_addr)?;
		Self::assert_open_port(internal_webgrpc_socket_addr)?;
		Self::assert_open_port(admin_webgrpc_socket_addr)?;

		Ok(EmbeddedServerWrapper {
			runtime,
			manager,
			game_socket_addr,
			internal_grpc_socket_addr,
			internal_webgrpc_socket_addr,
			admin_webgrpc_socket_addr,
		})
	}

	fn assert_open_port(socket: SocketAddr) -> Result<(), EmbeddedServerWrapperError> {
		let mut counter = 0;
		while !port_scanner::scan_port_addr(socket) {
			std::thread::sleep(Duration::from_millis(10));
			counter += 1;
			if counter > 100 {
				return Err(EmbeddedServerWrapperError::GrpcServicesNotStarted);
			}
		}
		Ok(())
	}

	pub fn shutdown(self) {
		let manager = Arc::clone(&self.manager);
		self.runtime.block_on(async move { manager.lock().await.shutdown() });
		self.runtime.shutdown_background();
	}
}

#[cfg(test)]
mod test {
	use std::net::{IpAddr, Ipv4Addr, SocketAddr};
	use std::time::Duration;

	use crate::EmbeddedServerWrapper;

	#[test]
	fn should_open_tcp_ports_after_start() {
		let server = create_server();
		let admin_grpc_port = server.admin_webgrpc_socket_addr.port();
		let internal_grpc_port = server.internal_grpc_socket_addr.port();
		assert!(port_scanner::scan_port(admin_grpc_port));
		assert!(port_scanner::scan_port(internal_grpc_port));
	}

	#[test]
	fn should_use_different_port_for_different_server() {
		let server_a = create_server();
		let server_b = create_server();
		assert_ne!(server_a.game_socket_addr, server_b.game_socket_addr);
		assert_ne!(server_a.admin_webgrpc_socket_addr, server_b.admin_webgrpc_socket_addr);
		assert_ne!(server_a.internal_grpc_socket_addr, server_b.internal_grpc_socket_addr);
	}

	#[test]
	fn should_shutdown_server() {
		let server = create_server();
		let admin_grpc_port = server.admin_webgrpc_socket_addr.port();
		let internal_grpc_port = server.internal_grpc_socket_addr.port();
		server.shutdown();
		std::thread::sleep(Duration::from_millis(100));
		assert!(!port_scanner::scan_port(admin_grpc_port));
		assert!(!port_scanner::scan_port(internal_grpc_port));
	}

	fn create_server() -> EmbeddedServerWrapper {
		let server = EmbeddedServerWrapper::run_new_server(
			SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
			SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
			SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
			SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
		)
		.unwrap();
		server
	}
}
