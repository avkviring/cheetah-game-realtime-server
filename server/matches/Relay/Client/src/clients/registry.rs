use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};
use std::thread;

use fnv::FnvBuildHasher;

use cheetah_matches_relay_common::network::client::ConnectionStatus;
use cheetah_matches_relay_common::room::{RoomId, RoomMemberId, UserPrivateKey};

use crate::clients::application_thread::ApplicationThreadClient;
use crate::clients::network_thread::NetworkThreadClient;

pub type ClientId = u16;

///
/// Реестр клиентов
///
/// - создание клиента/выполнение запросов от Unity/удаление клиента
/// - все методы Clients выполняются в главном потоке Unity
///
///
pub struct Registry {
	pub clients: HashMap<ClientId, ApplicationThreadClient, FnvBuildHasher>,
	client_generator_id: ClientId,
}

impl Default for Registry {
	fn default() -> Self {
		Registry {
			clients: Default::default(),
			client_generator_id: Default::default(),
		}
	}
}

impl Registry {
	pub fn create_client(
		&mut self,
		server_address: String,
		member_id: RoomMemberId,
		room_id: RoomId,
		user_private_key: UserPrivateKey,
		start_frame_id: u64,
	) -> std::io::Result<ClientId> {
		let start_frame_id = Arc::new(AtomicU64::new(start_frame_id));
		let state = Arc::new(Mutex::new(ConnectionStatus::Connecting));
		let state_cloned = state.clone();
		let rtt_in_ms = Arc::new(AtomicU64::new(0));
		let average_retransmit_frames = Arc::new(AtomicU32::new(0));

		let (sender, receiver) = std::sync::mpsc::channel();
		let (in_command_sender, in_command_receiver) = std::sync::mpsc::channel();
		let client = NetworkThreadClient::new(
			SocketAddr::from_str(server_address.as_str())
				.map_err(|e| std::io::Error::new(ErrorKind::AddrNotAvailable, format!("{:?}", e)))?,
			member_id,
			room_id,
			user_private_key,
			in_command_sender,
			state,
			receiver,
			start_frame_id.clone(),
			rtt_in_ms.clone(),
			average_retransmit_frames.clone(),
		)?;

		let handler = thread::Builder::new()
			.name(format!("user({:?})", member_id))
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
			start_frame_id,
			rtt_in_ms,
			average_retransmit_frames,
		);
		self.client_generator_id += 1;
		let client_id = self.client_generator_id;
		self.clients.insert(client_id, application_thread_client);

		log::info!("[registry] create client({})", client_id);
		Result::Ok(client_id)
	}

	pub fn destroy_client(&mut self, client: ClientId) -> Option<ApplicationThreadClient> {
		self.clients.remove(&client)
	}
}
