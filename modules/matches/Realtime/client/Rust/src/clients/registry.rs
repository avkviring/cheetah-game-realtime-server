use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{panic, thread};

use fnv::FnvBuildHasher;

use cheetah_matches_realtime_common::network::client::ConnectionStatus;
use cheetah_matches_realtime_common::room::{MemberPrivateKey, RoomId, RoomMemberId};

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::network_thread::NetworkThreadClient;
use crate::clients::SharedClientStatistics;

pub type ClientId = u16;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
///
#[derive(Default)]
pub struct Registry {
	pub clients: HashMap<ClientId, ApplicationThreadClient, FnvBuildHasher>,
	client_generator_id: ClientId,
}

impl Registry {
	pub fn create_client(
		&mut self,
		server_address: String,
		member_id: RoomMemberId,
		room_id: RoomId,
		user_private_key: MemberPrivateKey,
		start_frame_id: u64,
	) -> std::io::Result<ClientId> {
		Self::set_panic_hook();

		let server_time = Arc::new(Mutex::new(None));
		let state = Arc::new(Mutex::new(ConnectionStatus::Connecting));
		let state_cloned = state.clone();
		let shared_statistics = SharedClientStatistics::default();

		let (sender, receiver) = std::sync::mpsc::channel();
		let (in_command_sender, in_command_receiver) = std::sync::mpsc::channel();
		let client = NetworkThreadClient::new(
			SocketAddr::from_str(server_address.as_str()).map_err(|e| {
				std::io::Error::new(ErrorKind::AddrNotAvailable, format!("{:?}", e))
			})?,
			member_id,
			room_id,
			user_private_key,
			in_command_sender,
			state,
			receiver,
			start_frame_id,
			shared_statistics.clone(),
			server_time.clone(),
		)?;

		let handler = thread::Builder::new()
			.name(format!("member({:?})", member_id))
			.spawn(move || {
				client.run();
			})
			.unwrap();

		let application_thread_client = ApplicationThreadClient::new(
			member_id,
			handler,
			state_cloned,
			in_command_receiver,
			sender,
			shared_statistics,
			server_time,
		);
		self.client_generator_id += 1;
		let client_id = self.client_generator_id;
		self.clients.insert(client_id, application_thread_client);

		tracing::info!("[registry] create client({})", client_id);
		Ok(client_id)
	}

	fn set_panic_hook() {
		let default_panic = panic::take_hook();
		panic::set_hook(Box::new(move |panic_info| {
			let msg = format!("Panic in relay client {:?}", panic_info);
			thread::spawn(move || {
				tracing::error!("{}", msg);
			});
			thread::sleep(Duration::from_secs(2));
			default_panic(panic_info);
		}))
	}

	pub fn destroy_client(&mut self, client: ClientId) -> Option<ApplicationThreadClient> {
		self.clients.remove(&client)
	}
}
