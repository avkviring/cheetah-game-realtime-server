use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use rand::RngCore;
use rand::rngs::OsRng;
use stderrlog::Timestamp;

use cheetah_relay::server::Server;
use cheetah_relay_client::ffi::client::do_create_client;
use cheetah_relay_client::registry::ClientId;
use cheetah_relay_common::room::{RoomId, UserPrivateKey, UserPublicKey};
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::udp::bind_to_free_socket;

#[derive(Debug)]
pub struct Helper {
	room_id_generator: RoomId,
	user_public_key_generator: UserPublicKey,
}


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct UserKeys {
	pub public: UserPublicKey,
	pub private: UserPrivateKey,
}

impl Helper {
	pub fn new() -> Self {
		fn setup_logger() {
			stderrlog::new()
				.verbosity(0)
				.quiet(false)
				.show_level(true)
				.timestamp(Timestamp::Millisecond)
				.init();
		}
		Self {
			room_id_generator: 0,
			user_public_key_generator: 0,
		}
	}
	
	
	pub fn create_server_and_room(&mut self) -> (Server, SocketAddr, RoomId) {
		self.room_id_generator += 1;
		let room_id = self.room_id_generator;
		let binding = bind_to_free_socket().unwrap();
		let mut server = Server::new(binding.0, false);
		server.register_room(room_id).ok().unwrap();
		(server, binding.1, room_id)
	}
	
	
	pub fn create_user_keys(&mut self) -> UserKeys {
		self.user_public_key_generator += 1;
		let mut private_key = [0; 32];
		OsRng.fill_bytes(&mut private_key);
		UserKeys {
			public: self.user_public_key_generator,
			private: private_key,
		}
	}
	
	pub fn create_client(&self, address: &str, keys: UserKeys) -> ClientId {
		let mut client: ClientId = 0;
		do_create_client(address.to_string(), keys.public, &keys.private, 0, &mut client);
		client
	}
	
	pub fn setup_server_and_client(&mut self) -> (Server, ClientId) {
		let user_keys = self.create_user_keys();
		let (mut server, server_address, room_id) = self.create_server_and_room();
		server.register_user(room_id, user_keys.public, user_keys.private, AccessGroups(0b111)).ok().unwrap();
		let client = self.create_client(server_address.to_string().as_str(), user_keys);
		(server, client)
	}
	
	pub fn wait_first_frame(&self) {
		thread::sleep(Duration::from_millis(100));
	}
}


